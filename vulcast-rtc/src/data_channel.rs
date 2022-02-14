use std::{
    ffi::{CStr, CString},
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
};

use futures::Stream;
use thiserror::Error;
use tokio::sync::{
    broadcast,
    mpsc::{self, error::TrySendError},
    watch,
};

use crate::types::*;
use vulcast_rtc_sys as sys;

#[derive(Debug, Error)]
pub enum DataChannelError {
    #[error("channel is closed")]
    ChannelClosed,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DataChannelState {
    Connecting,
    Open,
    Closing,
    Closed,
}
impl FromStr for DataChannelState {
    type Err = ();
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "connecting" => Ok(DataChannelState::Connecting),
            "open" => Ok(DataChannelState::Open),
            "closing" => Ok(DataChannelState::Closing),
            "closed" => Ok(DataChannelState::Closed),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Data {
        data_consumer_id: DataConsumerId,
        data: Data,
    },
    DataConsumerStateChanged {
        data_consumer_id: DataConsumerId,
        state: DataChannelState,
    },
    DataProducerStateChanged {
        data_producer_id: DataProducerId,
        state: DataChannelState,
    },
}

pub type Data = Vec<u8>;

pub struct DataProducer {
    sys_data_producer: *mut sys::mediasoupclient_DataProducer,
    data_producer_id: DataProducerId,
    state: watch::Receiver<Option<DataChannelState>>,
}
unsafe impl Send for DataProducer {}
unsafe impl Sync for DataProducer {}
impl DataProducer {
    pub(crate) fn new(
        sys_broadcaster: *mut sys::Broadcaster,
        mut message_rx: broadcast::Receiver<Message>,
    ) -> Self {
        let sys_data_producer = unsafe { sys::data_producer_new(sys_broadcaster) };
        let data_producer_id_marshal = unsafe { sys::data_producer_marshal_id(sys_data_producer) };
        let data_producer_id = DataProducerId::from(unsafe {
            CStr::from_ptr(data_producer_id_marshal)
                .to_str()
                .unwrap()
                .to_owned()
        });
        unsafe { sys::cpp_unmarshal_str(data_producer_id_marshal) };
        let (state_tx, state_rx) = watch::channel(None);
        tokio::spawn({
            let data_producer_id = data_producer_id.clone();
            async move {
                loop {
                    tokio::select! {
                        Ok(message) = message_rx.recv() => {
                            match message {
                                Message::DataProducerStateChanged {
                                    data_producer_id: id,
                                    state: channel_state,
                                } if id == data_producer_id => {
                                    log::debug!("{:?}: state_changed {:?}", &id, &channel_state);
                                    if channel_state == DataChannelState::Closed {
                                        let _ = state_tx.send(Some(channel_state));
                                        return;
                                    }
                                }
                                _ => (),
                            }
                        },
                        _ = state_tx.closed() => {break},
                        else => {break}
                    }
                }
            }
        });
        Self {
            sys_data_producer,
            data_producer_id,
            state: state_rx,
        }
    }
    pub fn send(&mut self, data: Data) -> Result<(), DataChannelError> {
        let state = self.state.borrow_and_update();
        if let Some(DataChannelState::Closed) = *state {
            return Err(DataChannelError::ChannelClosed);
        }
        // this could potentially freeze the executor
        unsafe { sys::data_producer_send(self.sys_data_producer, data.as_ptr(), data.len() as u64) }
        Ok(())
    }
    pub fn id(&self) -> &DataProducerId {
        &self.data_producer_id
    }
}
impl Drop for DataProducer {
    fn drop(&mut self) {
        log::trace!("data producer delete {:?}", &self.sys_data_producer);
        unsafe {
            sys::data_producer_delete(self.sys_data_producer);
        }
    }
}

pub struct DataConsumer {
    sys_data_consumer: *mut sys::mediasoupclient_DataConsumer,
    data_consumer_id: DataConsumerId,
    data_rx: mpsc::Receiver<Data>,
}
unsafe impl Send for DataConsumer {}
unsafe impl Sync for DataConsumer {}
impl DataConsumer {
    pub(crate) fn new(
        sys_broadcaster: *mut sys::Broadcaster,
        data_consumer_options: DataConsumerOptions,
        mut message_rx: broadcast::Receiver<Message>,
    ) -> Self {
        let data_consumer_id = data_consumer_options.id;

        let (tx, rx) = mpsc::channel(16);

        tokio::spawn({
            let data_consumer_id = data_consumer_id.clone();
            async move {
                loop {
                    tokio::select! {
                        Ok(message) = message_rx.recv() => {
                            match message {
                                Message::Data {
                                    data_consumer_id: id,
                                    data,
                                } if id == data_consumer_id => {
                                    log::trace!("{:?}: data (len={:?})", &id, data.len());
                                    match tx.try_send(data) {
                                        Err(TrySendError::Closed(_)) => return,
                                        Err(TrySendError::Full(_)) => {
                                            log::warn!("{:?}: message dropped, you are reading stream too slowly", &id)
                                        },
                                        _ => {}
                                    }
                                }
                                Message::DataConsumerStateChanged {
                                    data_consumer_id: id,
                                    state: channel_state,
                                } if id == data_consumer_id => {
                                    log::debug!("{:?}: state_changed {:?}", &id, &channel_state);
                                    if channel_state == DataChannelState::Closed {
                                        return;
                                    }
                                }
                                _ => (),
                            }
                        },
                        _ = tx.closed() => {break},
                        else => {break}
                    }
                }
            }
        });

        let data_consumer_id_cstr = CString::new(String::from(data_consumer_id.clone())).unwrap();
        let data_producer_id_cstr =
            CString::new(String::from(data_consumer_options.data_producer_id)).unwrap();
        let sctp_stream_parameters_cstr = CString::new(
            serde_json::to_string(&data_consumer_options.sctp_stream_parameters).unwrap(),
        )
        .unwrap();
        let sys_data_consumer = unsafe {
            sys::data_consumer_new(
                sys_broadcaster,
                data_consumer_id_cstr.as_ptr(),
                data_producer_id_cstr.as_ptr(),
                sctp_stream_parameters_cstr.as_ptr(),
            )
        };
        log::trace!("data consumer new {:?}", &sys_data_consumer);
        Self {
            sys_data_consumer,
            data_consumer_id,
            data_rx: rx,
        }
    }

    pub fn id(&self) -> DataConsumerId {
        self.data_consumer_id.clone()
    }
}
impl Drop for DataConsumer {
    fn drop(&mut self) {
        log::trace!("data consumer delete {:?}", &self.sys_data_consumer);
        unsafe {
            sys::data_consumer_delete(self.sys_data_consumer);
        }
    }
}
impl Stream for DataConsumer {
    type Item = Data;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.data_rx.poll_recv(cx)
    }
}

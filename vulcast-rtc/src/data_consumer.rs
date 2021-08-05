use std::{
    ffi::CString,
    pin::Pin,
    str::FromStr,
    task::{Context, Poll},
};

use futures::Stream;
use tokio::sync::{broadcast, mpsc};

use crate::types::*;
use vulcast_rtc_sys as sys;

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
    StateChanged {
        data_consumer_id: DataConsumerId,
        state: DataChannelState,
    },
}

pub type Data = Vec<u8>;

pub struct DataConsumer {
    sys_data_consumer: *mut sys::mediasoupclient_DataConsumer,
    data_consumer_id: DataConsumerId,
    data_rx: mpsc::UnboundedReceiver<Data>,
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

        let (tx, rx) = mpsc::unbounded_channel();

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
                                    log::debug!("{:?}: data (len={:?})", &id, data.len());
                                    if let Err(_) = tx.send(data) {
                                        return;
                                    }
                                }
                                Message::StateChanged {
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

        unsafe {
            let data_consumer_id_cstr =
                CString::new(String::from(data_consumer_id.clone())).unwrap();
            let data_producer_id_cstr =
                CString::new(String::from(data_consumer_options.data_producer_id)).unwrap();
            let sctp_stream_parameters_cstr = CString::new(
                serde_json::to_string(&data_consumer_options.sctp_stream_parameters).unwrap(),
            )
            .unwrap();
            let sys_data_consumer = sys::data_consumer_new(
                sys_broadcaster,
                data_consumer_id_cstr.as_ptr(),
                data_producer_id_cstr.as_ptr(),
                sctp_stream_parameters_cstr.as_ptr(),
            );
            log::trace!("data consumer new {:?}", &sys_data_consumer);
            Self {
                sys_data_consumer,
                data_consumer_id,
                data_rx: rx,
            }
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

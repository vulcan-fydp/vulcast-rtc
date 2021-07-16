use std::{
    ffi::CString,
    str::FromStr,
    sync::{Arc, Mutex},
};

use futures::Stream;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;

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

#[derive(Clone)]
pub struct DataConsumer {
    shared: Arc<Shared>,
}
struct Shared {
    state: Mutex<State>,

    data_consumer_id: DataConsumerId,
    message_tx: broadcast::Sender<Message>,
}
unsafe impl Send for Shared {}
unsafe impl Sync for Shared {}
struct State {
    sys_data_consumer: *mut sys::mediasoupclient_DataConsumer,
    channel_state: DataChannelState,
}

impl DataConsumer {
    pub(crate) fn new(
        sys_broadcaster: *mut sys::Broadcaster,
        data_consumer_options: DataConsumerOptions,
        message_tx: broadcast::Sender<Message>,
    ) -> Self {
        unsafe {
            let data_consumer_id = data_consumer_options.id;
            let data_consumer_id_cstr =
                CString::new(String::from(data_consumer_id.clone())).unwrap();
            let data_producer_id_cstr =
                CString::new(String::from(data_consumer_options.data_producer_id)).unwrap();
            let sctp_stream_parameters_cstr = CString::new(
                serde_json::to_string(&data_consumer_options.sctp_stream_parameters).unwrap(),
            )
            .unwrap();

            // TODO robustness

            let sys_data_consumer = sys::data_consumer_new(
                sys_broadcaster,
                data_consumer_id_cstr.as_ptr(),
                data_producer_id_cstr.as_ptr(),
                sctp_stream_parameters_cstr.as_ptr(),
            );
            Self {
                shared: Arc::new(Shared {
                    state: Mutex::new(State {
                        sys_data_consumer,
                        channel_state: DataChannelState::Connecting,
                    }),
                    data_consumer_id,
                    message_tx,
                }),
            }
        }
    }

    pub fn id(&self) -> DataConsumerId {
        self.shared.data_consumer_id.clone()
    }

    pub fn stream(&self) -> impl Stream<Item = Data> {
        let mut message_rx = self.shared.message_tx.subscribe();
        let data_consumer_id = self.shared.data_consumer_id.clone();

        let (tx, rx) = mpsc::channel(16);
        let this = self.clone();
        tokio::spawn(async move {
            while let Ok(message) = message_rx.recv().await {
                match message {
                    Message::Data {
                        data_consumer_id: id,
                        data,
                    } if id == data_consumer_id => {
                        if let Err(_) = tx.send(data).await {
                            return;
                        }
                    }
                    Message::StateChanged {
                        data_consumer_id: id,
                        state: channel_state,
                    } if id == data_consumer_id => {
                        this.set_channel_state(channel_state);

                        if channel_state == DataChannelState::Closed {
                            return;
                        }
                    }
                    _ => (),
                }
            }
        });
        ReceiverStream::new(rx)
    }

    fn set_channel_state(&self, channel_state: DataChannelState) {
        let mut state = self.shared.state.lock().unwrap();
        state.channel_state = channel_state;
    }
}
impl Shared {
    fn get_sys_data_consumer(&self) -> *mut sys::mediasoupclient_DataConsumer {
        let state = self.state.lock().unwrap();
        state.sys_data_consumer
    }
}

impl Drop for Shared {
    fn drop(&mut self) {
        unsafe {
            sys::data_consumer_delete(self.get_sys_data_consumer());
        }
    }
}

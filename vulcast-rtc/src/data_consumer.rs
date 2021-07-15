use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

use futures::Stream;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;

use crate::types::*;
use vulcast_rtc_sys as sys;

#[derive(Debug, Clone)]
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
pub enum Message {
    Data {
        data_consumer_id: DataConsumerId,
        data: Vec<u8>,
    },
    StateChanged {
        data_consumer_id: DataConsumerId,
        state: DataChannelState,
    },
}

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
}

impl DataConsumer {
    pub fn new(
        sys_data_consumer: *mut sys::mediasoupclient_DataConsumer,
        data_consumer_id: DataConsumerId,
        message_tx: broadcast::Sender<Message>,
    ) -> Self {
        Self {
            shared: Arc::new(Shared {
                state: Mutex::new(State { sys_data_consumer }),
                data_consumer_id,
                message_tx,
            }),
        }
    }

    pub fn stream(&self) -> impl Stream<Item = Message> {
        let mut message_rx = self.shared.message_tx.subscribe();
        let data_consumer_id = self.shared.data_consumer_id.clone();

        let (tx, rx) = mpsc::channel(16);
        tokio::spawn(async move {
            while let Ok(message) = message_rx.recv().await {
                match &message {
                    Message::Data {
                        data_consumer_id: id,
                        ..
                    } if id == &data_consumer_id => {
                        if let Err(_) = tx.send(message).await {
                            return;
                        }
                    }
                    Message::StateChanged {
                        data_consumer_id: id,
                        state: DataChannelState::Closed,
                    } if id == &data_consumer_id => {
                        return;
                    }
                    _ => (),
                }
            }
        });
        ReceiverStream::new(rx)
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

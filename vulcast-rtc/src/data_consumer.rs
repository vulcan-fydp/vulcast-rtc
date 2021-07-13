use futures::Stream;
use tokio::{
    sync::{broadcast, mpsc},
};
use tokio_stream::wrappers::ReceiverStream;

use crate::types::*;
use vulcast_rtc_sys as sys;

#[derive(Debug, Clone)]
pub struct Message {
    pub data_consumer_id: DataConsumerId,
    pub data: Vec<u8>,
}

pub struct DataConsumer {
    sys_data_consumer: *mut sys::mediasoupclient_DataConsumer,
    data_consumer_id: DataConsumerId,
    message_tx: broadcast::Sender<Message>,
}

impl DataConsumer {
    pub fn new(
        sys_data_consumer: *mut sys::mediasoupclient_DataConsumer,
        data_consumer_id: DataConsumerId,
        message_tx: broadcast::Sender<Message>,
    ) -> Self {
        Self {
            sys_data_consumer,
            data_consumer_id,
            message_tx,
        }
    }

    pub fn stream(&self) -> impl Stream<Item = Message> {
        let mut message_rx = self.message_tx.subscribe();
        let data_consumer_id = self.data_consumer_id.clone();

        let (tx, rx) = mpsc::channel(16);
        tokio::spawn(async move {
            while let Ok(message) = message_rx.recv().await {
                match &message {
                    Message {
                        data_consumer_id: id,
                        ..
                    } => {
                        if id == &data_consumer_id {
                            if let Err(_) = tx.send(message).await {
                                return;
                            }
                        }
                    }
                }
            }
        });
        ReceiverStream::new(rx)
    }
}

impl Drop for DataConsumer {
    fn drop(&mut self) {
        unsafe {
            sys::stop_data_consumer(self.sys_data_consumer);
        }
    }
}

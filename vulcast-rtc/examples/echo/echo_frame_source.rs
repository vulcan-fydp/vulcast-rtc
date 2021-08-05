use font8x8::UnicodeFonts;
use futures::StreamExt;
use std::sync::{Arc, Mutex};
use tiny_skia::{Color, Paint, PixmapMut, Rect, Transform};
use tokio::sync::{mpsc, oneshot};

use graphql_ws::GraphQLOperation;
use vulcast_rtc::{broadcaster::WeakBroadcaster, frame_source::FrameSource};

use crate::{controller_message::*, signal_schema::DataProducerAvailable};

#[derive(Clone)]
pub struct EchoFrameSource {
    shared: Arc<Shared>,
}
struct Shared {
    state: Mutex<State>,
}
struct State {
    last_message: ControllerMessage,
}
impl EchoFrameSource {
    pub fn new(
        weak_broadcaster: WeakBroadcaster,
        data_producer_available: GraphQLOperation<DataProducerAvailable>,
    ) -> Self {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                last_message: ControllerMessage::default(),
            }),
        });
        let mut data_producer_available_stream = data_producer_available.execute();
        let weak_shared = Arc::downgrade(&shared);
        tokio::spawn(async move {
            let (message_tx, mut message_rx) = mpsc::unbounded_channel::<Vec<u8>>();
            loop {
                tokio::select! {
                    Some(msg) = message_rx.recv() => {
                        let msg = ControllerMessage::from_slice_u8(&msg);
                        if let Ok(msg) = msg {
                            let shared = weak_shared.upgrade()?;
                            let mut state = shared.state.lock().unwrap();
                            state.last_message = msg;
                        } else {
                            println!("rejected malformed message");
                        }
                    },
                    Some(Ok(response)) = data_producer_available_stream.next() => {
                        let broadcaster = weak_broadcaster.upgrade()?;
                        let data_producer_id = response.data.unwrap().data_producer_available;
                        println!("{:?}: data producer available", &data_producer_id);
                        let mut data_consumer = broadcaster.consume_data(data_producer_id).await;
                        tokio::spawn(enclose! { (message_tx) async move {
                            while let Some(msg) = data_consumer.next().await {
                                let _ = message_tx.send(msg);
                            }
                        }});
                    },
                    else => {break}
                };
            }
            Some::<()>(())
        });
        EchoFrameSource { shared }
    }
}
impl FrameSource for EchoFrameSource {
    fn next_frame(&self, width: u32, height: u32, timestamp: i64, data: &mut [u8]) {
        let mut pixmap = PixmapMut::from_bytes(data, width, height).unwrap();
        let mut paint = Paint::default();
        paint.set_color_rgba8(255, 255, 255, 255);
        pixmap.fill(Color::BLACK);
        let x = ((timestamp / 10000) as u32 % width) as f32;
        let y = ((timestamp / 10000) as u32 % height) as f32;
        pixmap.fill_rect(
            Rect::from_xywh(x, y, 10.0, 10.0).unwrap(),
            &paint,
            Transform::identity(),
            None,
        );

        let state = self.shared.state.lock().unwrap();
        let msg_dump = format!("{:#?}", &state.last_message);

        blit_text(
            format!("Last Message Received:\n\n{}", &msg_dump).as_str(),
            10,
            10,
            data,
            width,
            [255, 255, 255, 255],
        );
    }
}

fn blit_text(text: &str, x: u32, y: u32, data: &mut [u8], width: u32, color: [u8; 4]) {
    const BPP: usize = 4;
    let stride = width as usize * BPP;
    let mut ax = x as usize;
    let mut ay = y as usize;
    for c in text.chars() {
        if c == '\n' {
            ax = x as usize;
            ay += 8;
            continue;
        }
        let bitmap = font8x8::BASIC_FONTS.get(c).unwrap();
        for row in 0..8 {
            for col in 0..8 {
                if bitmap[row] & 1 << col != 0 {
                    let cursor = (ay + row) * stride + (ax + col) * BPP;
                    if cursor + 4 >= data.len() {
                        return;
                    }
                    data[cursor] = color[0];
                    data[cursor + 1] = color[1];
                    data[cursor + 2] = color[2];
                    data[cursor + 3] = color[3];
                }
            }
        }
        ax += 8;
    }
}

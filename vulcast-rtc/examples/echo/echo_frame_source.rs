use font8x8::UnicodeFonts;
use futures::StreamExt;
use std::sync::{Arc, Mutex};
use tiny_skia::{Color, Paint, PixmapMut, Rect, Transform};
use tokio::sync::mpsc;

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
    p: [f32; 2],
    v: [f32; 2],
    s: [f32; 2],
}
impl EchoFrameSource {
    pub fn new(
        weak_broadcaster: WeakBroadcaster,
        data_producer_available: GraphQLOperation<DataProducerAvailable>,
    ) -> Self {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                last_message: ControllerMessage::default(),
                p: [10.0, 10.0],
                v: [5.0, 3.0],
                s: [50.0, 50.0],
            }),
        });
        let mut data_producer_available_stream = data_producer_available.execute();
        let weak_shared = Arc::downgrade(&shared);
        tokio::spawn(async move {
            let (message_tx, mut message_rx) = mpsc::unbounded_channel::<Vec<u8>>();
            loop {
                tokio::select! {
                    Some(msg) = message_rx.recv() => {
                        println!("{:?}", msg);
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

        let mut state = self.shared.state.lock().unwrap();
        let rect = Rect::from_xywh(state.p[0], state.p[1], state.s[0], state.s[1]).unwrap();
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);

        // shitty physics
        let dim = [width, height];
        for i in 0..=1 {
            state.p[i] += state.v[i];
            if state.p[i] + state.s[i] > dim[i] as f32 || state.p[i] < 0.0 {
                state.v[i] *= -1.0;
            }
        }

        blit_text(
            format!(
                "vulcast-rtc now={}\n\nLast Message Received:\n\n{:#?}",
                timestamp, &state.last_message
            )
            .as_str(),
            10,
            10,
            data,
            width,
        );
    }
}

fn blit_text(text: &str, x: u32, y: u32, data: &mut [u8], width: u32) {
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
                    data[cursor] = 255 - data[cursor];
                    data[cursor + 1] = 255 - data[cursor + 1];
                    data[cursor + 2] = 255 - data[cursor + 2];
                    // data[cursor + 3] = color[3];
                }
            }
        }
        ax += 8;
    }
}

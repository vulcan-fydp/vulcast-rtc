use std::os::raw::c_ulong;
use std::ptr;
use std::str::FromStr;
use std::sync::{Arc, Mutex, Weak};
use std::{
    ffi::{c_void, CStr, CString},
    os::raw::c_char,
};

use async_trait::async_trait;
use tokio::sync::{broadcast, mpsc};

use crate::alsa_capturer::AlsaCapturer;
use crate::data_channel::{self, DataConsumer, DataProducer};
use crate::foreign_producer::ForeignProducer;
use crate::frame_source::FrameSource;
use crate::types::*;
use crate::vcm_capturer::{VcmCapturer, VideoType};
use vulcast_rtc_sys as sys;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TransportConnectionState {
    Closed,
    Failed,
    Disconnected,
    New,
    Connecting,
    Connected,
    // not actually part of RTCPeerConnectionState
    Checking,
    Completed,
}
impl FromStr for TransportConnectionState {
    type Err = ();
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "closed" => Ok(TransportConnectionState::Closed),
            "failed" => Ok(TransportConnectionState::Failed),
            "disconnected" => Ok(TransportConnectionState::Disconnected),
            "new" => Ok(TransportConnectionState::New),
            "connecting" => Ok(TransportConnectionState::Connecting),
            "connected" => Ok(TransportConnectionState::Connected),
            "checking" => Ok(TransportConnectionState::Checking),
            "completed" => Ok(TransportConnectionState::Completed),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
enum InternalMessage {
    TransportConnectionStateChanged {
        transport_id: TransportId,
        state: TransportConnectionState,
    },
}

#[derive(Clone)]
pub struct Broadcaster {
    shared: Arc<Shared>,
}
struct Shared {
    state: Mutex<State>,
    signaller: Arc<dyn Signaller>,

    data_channel_tx: broadcast::Sender<data_channel::Message>,
    channel_tx: mpsc::UnboundedSender<InternalMessage>,
}
unsafe impl Send for Shared {}
unsafe impl Sync for Shared {}
struct State {
    sys_broadcaster: *mut sys::Broadcaster,
}

#[derive(Clone)]
pub struct WeakBroadcaster {
    shared: Weak<Shared>,
}

#[async_trait]
pub trait Signaller: Send + Sync {
    async fn server_rtp_capabilities(&self) -> RtpCapabilitiesFinalized;
    async fn create_webrtc_transport(&self) -> WebRtcTransportOptions;
    async fn on_rtp_capabilities(&self, rtp_caps: RtpCapabilities);
    async fn on_produce(
        &self,
        transport_id: TransportId,
        kind: MediaKind,
        rtp_parameters: RtpParameters,
    ) -> ProducerId;
    async fn on_produce_data(
        &self,
        transport_id: TransportId,
        sctp_stream_parameters: SctpStreamParameters,
    ) -> DataProducerId;
    async fn on_connect_webrtc_transport(
        &self,
        transport_id: TransportId,
        dtls_parameters: DtlsParameters,
    );
    async fn consume_data(
        &self,
        transport_id: TransportId,
        data_producer_id: DataProducerId,
    ) -> Result<DataConsumerOptions, Box<dyn std::error::Error>>;
    async fn on_connection_state_changed(
        &self,
        transport_id: TransportId,
        state: TransportConnectionState,
    );
}

impl Broadcaster {
    /// Create a new broadcaster with the given signalling handlers.
    pub async fn new(signaller: Arc<dyn Signaller>) -> Self {
        super::native_init();

        let (channel_tx, mut channel_rx) = mpsc::unbounded_channel();
        let shared = tokio::task::spawn_blocking({
            let signaller = signaller.clone();
            move || {
                let shared = Arc::new(Shared {
                    state: Mutex::new(State {
                        sys_broadcaster: ptr::null_mut(),
                    }),
                    signaller,
                    data_channel_tx: broadcast::channel(16).0,
                    channel_tx,
                });
                let sys_broadcaster = unsafe {
                    sys::broadcaster_new(
                        &*shared as *const _ as *const c_void,
                        sys::SignalHandler {
                            server_rtp_capabilities: Some(server_rtp_capabilities),
                            on_rtp_capabilities: Some(on_rtp_capabilities),
                            on_produce: Some(on_produce),
                            on_produce_data: Some(on_produce_data),
                            on_connect_webrtc_transport: Some(on_connect_webrtc_transport),
                            create_webrtc_transport: Some(create_webrtc_transport),
                            on_data_consumer_message: Some(on_data_consumer_message),
                            on_data_consumer_state_changed: Some(on_data_consumer_state_changed),
                            on_data_producer_state_changed: Some(on_data_producer_state_changed),
                            on_connection_state_changed: Some(on_connection_state_changed),
                        },
                    )
                };
                log::trace!("broadcaster new {:?}", sys_broadcaster);
                let mut state = shared.state.lock().unwrap();
                state.sys_broadcaster = sys_broadcaster;
                drop(state);
                shared
            }
        })
        .await
        .unwrap();
        tokio::spawn(async move {
            while let Some(message) = channel_rx.recv().await {
                match message {
                    InternalMessage::TransportConnectionStateChanged {
                        transport_id,
                        state,
                    } => {
                        signaller
                            .on_connection_state_changed(transport_id, state)
                            .await;
                    }
                }
            }
        });
        Self { shared }
    }

    /// Consume data from the given data producer.
    pub async fn consume_data(
        &self,
        data_producer_id: DataProducerId,
    ) -> Result<DataConsumer, Box<dyn std::error::Error>> {
        let recv_transport_id = self.get_recv_transport_id();

        let data_consumer_options = self
            .shared
            .signaller
            .consume_data(recv_transport_id, data_producer_id.clone())
            .await?;

        // spawn on blocking thread
        let data_consumer = tokio::task::spawn_blocking({
            let broadcaster = self.clone();
            move || {
                let sys = broadcaster.sys();
                let data_consumer_rx = broadcaster.shared.data_channel_tx.subscribe();
                DataConsumer::new(sys, data_consumer_options, data_consumer_rx)
            }
        })
        .await
        .unwrap();
        Ok(data_consumer)
    }

    /// Produce data on send transport.
    pub async fn produce_data(&self) -> DataProducer {
        // spawn on blocking thread
        let data_producer = tokio::task::spawn_blocking({
            let broadcaster = self.clone();
            move || {
                let sys = broadcaster.sys();
                let data_producer_rx = broadcaster.shared.data_channel_tx.subscribe();
                DataProducer::new(sys, data_producer_rx)
            }
        })
        .await
        .unwrap();
        data_producer
    }

    /// Produce a fake media stream for debugging purposes (leaks memory).
    // pub fn debug_produce_fake_media(&self) {
    //     // deadlock risk
    //     unsafe {
    //         sys::producer_new_from_fake_audio(self.sys());
    //         sys::producer_new_from_fake_video(self.sys());
    //     }
    // }

    /// Produce a fake media stream from the first available video device for
    /// debugging purposes (leaks memory).
    // pub fn debug_produce_video_from_vcm_capturer(&self) {
    //     // deadlock risk
    //     unsafe {
    //         sys::producer_new_from_vcm_capturer(self.sys());
    //     }
    // }

    // Produce an audio stream using AlsaCapturer, from the default ALSA device.
    pub async fn produce_audio_from_default_alsa(&self) -> AlsaCapturer {
        // spawn on blocking thread
        tokio::task::spawn_blocking({
            let broadcaster = self.clone();
            move || {
                let sys = broadcaster.sys();
                AlsaCapturer::new(sys)
            }
        })
        .await
        .unwrap()
    }

    /// Produce a video stream using VcmCapturer, allowing us to capture from
    /// any video device (e.g. webcam, capture card). The video device must
    /// support the given width, height, and FPS with the specified video
    /// format. You can query the capabilities of your video device with
    /// `v4l2-ctl -d /dev/videoX --list-formats-ext`.
    pub async fn produce_video_from_vcm_capturer_with_format(
        &self,
        device_idx: Option<i32>,
        width: u32,
        height: u32,
        fps: u32,
        video_type: VideoType,
    ) -> VcmCapturer {
        // spawn on blocking thread
        tokio::task::spawn_blocking({
            let broadcaster = self.clone();
            move || {
                let sys = broadcaster.sys();
                VcmCapturer::new(sys, device_idx, width, height, fps, video_type)
            }
        })
        .await
        .unwrap()
    }

    pub async fn produce_video_from_vcm_capturer(
        &self,
        device_idx: Option<i32>,
        width: u32,
        height: u32,
        fps: u32,
    ) -> VcmCapturer {
        self.produce_video_from_vcm_capturer_with_format(
            device_idx,
            width,
            height,
            fps,
            VideoType::MJPEG,
        )
        .await
    }

    /// Produce a video stream from a programatically generated source.
    /// The provided frame source is initialized with the provided dimensions,
    /// and will be polled from a RTC thread at the given frame rate.
    pub async fn produce_video_from_frame_source(
        &self,
        frame_source: Arc<dyn FrameSource>,
        width: u32,
        height: u32,
        fps: u32,
    ) -> ForeignProducer {
        // spawn on blocking thread
        tokio::task::spawn_blocking({
            let broadcaster = self.clone();
            move || {
                let sys = broadcaster.sys();
                ForeignProducer::new(sys, frame_source, width, height, fps)
            }
        })
        .await
        .unwrap()
    }

    fn sys(&self) -> *mut sys::Broadcaster {
        let state = self.shared.state.lock().unwrap();
        state.sys_broadcaster
    }

    pub fn downgrade(&self) -> WeakBroadcaster {
        WeakBroadcaster {
            shared: Arc::downgrade(&self.shared),
        }
    }

    fn get_recv_transport_id(&self) -> TransportId {
        unsafe {
            let recv_transport_id_marshal = sys::broadcaster_marshal_recv_transport_id(self.sys());
            let recv_transport_id = TransportId::from(
                CStr::from_ptr(recv_transport_id_marshal)
                    .to_str()
                    .unwrap()
                    .to_owned(),
            );
            sys::cpp_unmarshal_str(recv_transport_id_marshal);
            recv_transport_id
        }
    }
}
impl WeakBroadcaster {
    pub fn upgrade(&self) -> Option<Broadcaster> {
        let shared = self.shared.upgrade()?;
        Some(Broadcaster { shared })
    }
}

impl Drop for Shared {
    fn drop(&mut self) {
        let state = self.state.lock().unwrap();
        log::trace!("broadcaster delete {:?}", state.sys_broadcaster);
        unsafe { sys::broadcaster_delete(state.sys_broadcaster) }
    }
}

extern "C" fn server_rtp_capabilities(ctx: *const c_void) -> *mut c_char {
    log::trace!("server_rtp_capabilities({:?})", ctx);
    let shared = unsafe { &*(ctx as *const Shared) };

    let (tx, mut rx) = mpsc::channel(1);
    let fut = shared.signaller.server_rtp_capabilities();
    tokio::spawn(async move {
        tx.send(fut.await).await.unwrap();
    });

    let server_rtp_capabilities = rx.blocking_recv().unwrap();
    CString::new(serde_json::to_string(&server_rtp_capabilities).unwrap())
        .unwrap()
        .into_raw()
}
extern "C" fn create_webrtc_transport(ctx: *const c_void) -> *mut c_char {
    log::trace!("create_webrtc_transport({:?})", ctx);
    let shared = unsafe { &*(ctx as *const Shared) };
    let (tx, mut rx) = mpsc::channel(1);
    let fut = shared.signaller.create_webrtc_transport();
    tokio::spawn(async move {
        tx.send(fut.await).await.unwrap();
    });

    let webrtc_transport_options = rx.blocking_recv().unwrap();
    CString::new(serde_json::to_string(&webrtc_transport_options).unwrap())
        .unwrap()
        .into_raw()
}
extern "C" fn on_rtp_capabilities(ctx: *const c_void, rtp_caps: *const c_char) {
    log::trace!("on_rtp_capabilities({:?})", ctx);
    let shared = unsafe { &*(ctx as *const Shared) };
    let rtp_caps = unsafe { CStr::from_ptr(rtp_caps).to_str().unwrap() };

    let (tx, mut rx) = mpsc::channel(1);
    let fut = shared.signaller.on_rtp_capabilities(RtpCapabilities::from(
        serde_json::from_str::<serde_json::Value>(rtp_caps).unwrap(),
    ));
    tokio::spawn(async move { tx.send(fut.await).await.unwrap() });
    let _ = rx.blocking_recv().unwrap();
}
extern "C" fn on_produce(
    ctx: *const c_void,
    transport_id: *const c_char,
    kind: *const c_char,
    rtp_parameters: *const c_char,
) -> *mut c_char {
    log::trace!("on_produce({:?})", ctx);
    unsafe {
        let shared = &*(ctx as *const Shared);
        let transport_id_cstr = CStr::from_ptr(transport_id);
        let kind_cstr = CStr::from_ptr(kind);
        let rtp_parameters = CStr::from_ptr(rtp_parameters).to_str().unwrap();

        let (tx, mut rx) = mpsc::channel(1);
        let fut = shared.signaller.on_produce(
            TransportId::from(transport_id_cstr.to_str().unwrap().to_owned()),
            MediaKind::from_str(kind_cstr.to_string_lossy().as_ref()).unwrap(),
            RtpParameters::from(serde_json::from_str::<serde_json::Value>(rtp_parameters).unwrap()),
        );
        tokio::spawn(async move {
            tx.send(fut.await).await.unwrap();
        });

        let producer_id = rx.blocking_recv().unwrap();
        CString::new(String::from(producer_id)).unwrap().into_raw()
    }
}
extern "C" fn on_produce_data(
    ctx: *const c_void,
    transport_id: *const c_char,
    sctp_stream_parameters: *const c_char,
) -> *mut c_char {
    log::trace!("on_produce_data({:?})", ctx);
    unsafe {
        let shared = &*(ctx as *const Shared);
        let transport_id_cstr = CStr::from_ptr(transport_id);
        let sctp_stream_parameters = CStr::from_ptr(sctp_stream_parameters).to_str().unwrap();

        let (tx, mut rx) = mpsc::channel(1);
        let fut = shared.signaller.on_produce_data(
            TransportId::from(transport_id_cstr.to_str().unwrap().to_owned()),
            SctpStreamParameters::from(
                serde_json::from_str::<serde_json::Value>(sctp_stream_parameters).unwrap(),
            ),
        );
        tokio::spawn(async move {
            tx.send(fut.await).await.unwrap();
        });

        let producer_id = rx.blocking_recv().unwrap();
        CString::new(String::from(producer_id)).unwrap().into_raw()
    }
}
extern "C" fn on_connect_webrtc_transport(
    ctx: *const c_void,
    transport_id: *const c_char,
    dtls_parameters: *const c_char,
) {
    log::trace!("on_connect_webrtc_transport({:?})", ctx);
    unsafe {
        let shared = &*(ctx as *const Shared);
        let transport_id_cstr = CStr::from_ptr(transport_id);
        let dtls_parameters = CStr::from_ptr(dtls_parameters).to_str().unwrap();

        let (tx, mut rx) = mpsc::channel(1);
        let fut = shared.signaller.on_connect_webrtc_transport(
            TransportId::from(transport_id_cstr.to_str().unwrap().to_owned()),
            DtlsParameters::from(
                serde_json::from_str::<serde_json::Value>(dtls_parameters).unwrap(),
            ),
        );
        tokio::spawn(async move { tx.send(fut.await).await.unwrap() });
        let _ = rx.blocking_recv().unwrap();
    }
}
extern "C" fn on_data_consumer_message(
    ctx: *const c_void,
    data_consumer_id: *const c_char,
    data: *const c_char,
    len: c_ulong,
) {
    log::trace!("on_data_consumer_message({:?}, len={})", ctx, len);
    unsafe {
        let shared = &*(ctx as *const Shared);
        let data_consumer_id_cstr = CStr::from_ptr(data_consumer_id);
        let message_data = std::slice::from_raw_parts(data as *const u8, len as usize).to_vec();
        let _ = shared.data_channel_tx.send(data_channel::Message::Data {
            data_consumer_id: DataConsumerId::from(
                data_consumer_id_cstr.to_str().unwrap().to_owned(),
            ),
            data: message_data,
        });
    }
}
extern "C" fn on_data_consumer_state_changed(
    ctx: *const c_void,
    data_consumer_id: *const c_char,
    state: *const c_char,
) {
    log::trace!("on_data_consumer_state_changed({:?})", ctx);
    unsafe {
        let shared = &*(ctx as *const Shared);
        let data_consumer_id_cstr = CStr::from_ptr(data_consumer_id);
        let state_cstr = CStr::from_ptr(state);
        let _ = shared
            .data_channel_tx
            .send(data_channel::Message::DataConsumerStateChanged {
                data_consumer_id: DataConsumerId::from(
                    data_consumer_id_cstr.to_str().unwrap().to_owned(),
                ),
                state: data_channel::DataChannelState::from_str(state_cstr.to_str().unwrap())
                    .unwrap(),
            });
    }
}
extern "C" fn on_data_producer_state_changed(
    ctx: *const c_void,
    data_producer_id: *const c_char,
    state: *const c_char,
) {
    log::trace!("on_data_producer_state_changed({:?})", ctx);
    unsafe {
        let shared = &*(ctx as *const Shared);
        let data_producer_id_cstr = CStr::from_ptr(data_producer_id);
        let state_cstr = CStr::from_ptr(state);
        let _ = shared
            .data_channel_tx
            .send(data_channel::Message::DataProducerStateChanged {
                data_producer_id: DataProducerId::from(
                    data_producer_id_cstr.to_str().unwrap().to_owned(),
                ),
                state: data_channel::DataChannelState::from_str(state_cstr.to_str().unwrap())
                    .unwrap(),
            });
    }
}
extern "C" fn on_connection_state_changed(
    ctx: *const c_void,
    transport_id: *const c_char,
    state: *const c_char,
) {
    log::trace!("on_connection_state_changed({:?})", ctx);
    unsafe {
        let shared = &*(ctx as *const Shared);
        let transport_id_cstr = CStr::from_ptr(transport_id);
        let state_cstr = CStr::from_ptr(state);

        let _ = shared
            .channel_tx
            .send(InternalMessage::TransportConnectionStateChanged {
                transport_id: TransportId::from(transport_id_cstr.to_str().unwrap().to_owned()),
                state: TransportConnectionState::from_str(state_cstr.to_str().unwrap()).unwrap(),
            });
    }
}

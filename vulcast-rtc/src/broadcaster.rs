use futures::future::BoxFuture;
use std::os::raw::c_ulong;
use std::pin::Pin;
use std::ptr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::{
    ffi::{c_void, CStr, CString},
    os::raw::c_char,
};

use lazy_static::lazy_static;
use tokio::runtime::{self, Runtime};
use tokio::sync::broadcast;

use crate::data_consumer::{self, DataConsumer};
use crate::foreign_producer::ForeignProducer;
use crate::frame_source::FrameSource;
use crate::types::*;
use vulcast_rtc_sys as sys;

lazy_static! {
    static ref NATIVE_RUNTIME: Runtime = runtime::Builder::new_multi_thread().build().unwrap();
}

#[derive(Clone)]
pub struct Broadcaster {
    shared: Pin<Arc<Shared>>,
}
struct Shared {
    state: Mutex<State>,
    handlers: Handlers,

    data_consumer_tx: broadcast::Sender<data_consumer::Message>,
}
unsafe impl Send for Shared {}
unsafe impl Sync for Shared {}
struct State {
    sys_broadcaster: *mut sys::Broadcaster,
}

pub struct Handlers {
    pub server_rtp_capabilities:
        Box<dyn Fn() -> BoxFuture<'static, RtpCapabilitiesFinalized> + Send + Sync + 'static>,
    pub create_webrtc_transport:
        Box<dyn Fn() -> BoxFuture<'static, WebRtcTransportOptions> + Send + Sync + 'static>,
    pub on_rtp_capabilities:
        Box<dyn Fn(RtpCapabilities) -> BoxFuture<'static, ()> + Send + Sync + 'static>,
    pub on_produce: Box<
        dyn Fn(TransportId, MediaKind, RtpParameters) -> BoxFuture<'static, ProducerId>
            + Send
            + Sync
            + 'static,
    >,
    pub on_connect_webrtc_transport:
        Box<dyn Fn(TransportId, DtlsParameters) -> BoxFuture<'static, ()> + Send + Sync + 'static>,
    pub consume_data: Box<
        dyn Fn(TransportId, DataProducerId) -> BoxFuture<'static, DataConsumerOptions>
            + Send
            + Sync
            + 'static,
    >,
}

impl Broadcaster {
    /// Create a new broadcaster with the given signalling handlers.
    pub fn new(handlers: Handlers) -> Self {
        super::native_init();
        let shared = Arc::pin(Shared {
            state: Mutex::new(State {
                sys_broadcaster: ptr::null_mut(),
            }),
            handlers,
            data_consumer_tx: broadcast::channel(16).0,
        });
        unsafe {
            let sys_broadcaster = sys::broadcaster_new(
                &*shared as *const _ as *const c_void,
                sys::SignalHandler {
                    server_rtp_capabilities: Some(server_rtp_capabilities),
                    on_rtp_capabilities: Some(on_rtp_capabilities),
                    on_produce: Some(on_produce),
                    on_connect_webrtc_transport: Some(on_connect_webrtc_transport),
                    consume_data: Some(consume_data),
                    create_webrtc_transport: Some(create_webrtc_transport),
                    on_data_consumer_message: Some(on_data_consumer_message),
                    on_data_consumer_state_changed: Some(on_data_consumer_state_changed),
                },
            );
            log::trace!("broadcaster new {:?}", sys_broadcaster);
            let mut state = shared.state.lock().unwrap();
            state.sys_broadcaster = sys_broadcaster;
        }
        Self { shared }
    }

    fn get_recv_transport_id(&self) -> TransportId {
        unsafe {
            let recv_transport_id_ptr = sys::broadcaster_get_recv_transport_id(self.sys());
            let recv_transport_id = TransportId::from(
                CStr::from_ptr(recv_transport_id_ptr)
                    .to_str()
                    .unwrap()
                    .to_owned(),
            );
            sys::delete_str(recv_transport_id_ptr);
            recv_transport_id
        }
    }

    /// Consume data from the given data producer.
    pub async fn consume_data(&self, data_producer_id: DataProducerId) -> DataConsumer {
        unsafe {
            let recv_transport_id = self.get_recv_transport_id();

            let data_consumer_options =
                (self.shared.handlers.consume_data)(recv_transport_id, data_producer_id.clone())
                    .await;

            let data_consumer = DataConsumer::new(
                self.sys(),
                data_consumer_options,
                self.shared.data_consumer_tx.clone(),
            );
            data_consumer
        }
    }

    /// Produce a fake media stream for debugging purposes (leaks memory).
    pub fn debug_produce_fake_media(&self) {
        unsafe {
            sys::producer_new_from_fake_audio(self.sys());
            sys::producer_new_from_fake_video(self.sys());
        }
    }

    /// Produce a fake media stream from the first available video device for
    /// debugging purposes (leaks memory).
    pub fn debug_produce_video_from_vcm_capturer(&self) {
        unsafe {
            sys::producer_new_from_vcm_capturer(self.sys());
        }
    }

    /// Produce a video stream from a programatically generated source.
    /// The provided frame source is initialized with the provided dimensions,
    /// and will be polled from a RTC thread at the given frame rate.
    pub fn produce_video_from_frame_source(
        &self,
        frame_source: Arc<dyn FrameSource>,
        width: u32,
        height: u32,
        fps: u32,
    ) -> ForeignProducer {
        ForeignProducer::new(self.sys(), frame_source, width, height, fps)
    }

    fn sys(&self) -> *mut sys::Broadcaster {
        let state = self.shared.state.lock().unwrap();
        state.sys_broadcaster
    }
}

impl Drop for State {
    fn drop(&mut self) {
        log::trace!("broadcaster delete {:?}", self.sys_broadcaster);
        unsafe { sys::broadcaster_delete(self.sys_broadcaster) }
    }
}

extern "C" fn server_rtp_capabilities(ctx: *const c_void) -> *mut c_char {
    log::trace!("server_rtp_capabilities({:?})", ctx);
    let shared = unsafe { &*(ctx as *const Shared) };

    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let fut = (shared.handlers.server_rtp_capabilities)();
    NATIVE_RUNTIME.spawn(async move {
        let _ = tx.send(fut.await);
    });

    let server_rtp_capabilities = rx.recv().unwrap();
    CString::new(serde_json::to_string(&server_rtp_capabilities).unwrap())
        .unwrap()
        .into_raw()
}
extern "C" fn create_webrtc_transport(ctx: *const c_void) -> *mut c_char {
    log::trace!("create_webrtc_transport({:?})", ctx);
    let shared = unsafe { &*(ctx as *const Shared) };
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let fut = (shared.handlers.create_webrtc_transport)();
    NATIVE_RUNTIME.spawn(async move {
        let _ = tx.send(fut.await);
    });

    let webrtc_transport_options = rx.recv().unwrap();
    CString::new(serde_json::to_string(&webrtc_transport_options).unwrap())
        .unwrap()
        .into_raw()
}
extern "C" fn on_rtp_capabilities(ctx: *const c_void, rtp_caps: *const c_char) {
    log::trace!("on_rtp_capabilities({:?})", ctx);
    let shared = unsafe { &*(ctx as *const Shared) };
    let rtp_caps = unsafe { CStr::from_ptr(rtp_caps).to_str().unwrap() };

    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let fut = (shared.handlers.on_rtp_capabilities)(RtpCapabilities::from(
        serde_json::from_str::<serde_json::Value>(rtp_caps).unwrap(),
    ));
    NATIVE_RUNTIME.spawn(async move {
        let _ = tx.send(fut.await);
    });
    let _ = rx.recv().unwrap();
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

        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let fut = (shared.handlers.on_produce)(
            transport_id_cstr.to_string_lossy().into_owned().into(),
            MediaKind::from_str(kind_cstr.to_string_lossy().as_ref()).unwrap(),
            RtpParameters::from(serde_json::from_str::<serde_json::Value>(rtp_parameters).unwrap()),
        );
        NATIVE_RUNTIME.spawn(async move {
            let _ = tx.send(fut.await);
        });

        let producer_id = rx.recv().unwrap();
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

        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let fut = (shared.handlers.on_connect_webrtc_transport)(
            transport_id_cstr.to_string_lossy().into_owned().into(),
            DtlsParameters::from(
                serde_json::from_str::<serde_json::Value>(dtls_parameters).unwrap(),
            ),
        );
        NATIVE_RUNTIME.spawn(async move {
            let _ = tx.send(fut.await);
        });

        let _ = rx.recv().unwrap();
    }
}
extern "C" fn consume_data(
    ctx: *const c_void,
    transport_id: *const c_char,
    data_producer_id: *const c_char,
) -> *mut c_char {
    log::trace!("consume_data({:?})", ctx);
    unsafe {
        let shared = &*(ctx as *const Shared);
        let transport_id_cstr = CStr::from_ptr(transport_id);
        let data_producer_id_cstr = CStr::from_ptr(data_producer_id);

        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let fut = (shared.handlers.consume_data)(
            transport_id_cstr.to_string_lossy().into_owned().into(),
            data_producer_id_cstr.to_string_lossy().into_owned().into(),
        );
        NATIVE_RUNTIME.spawn(async move {
            let _ = tx.send(fut.await);
        });

        let data_consumer_options = rx.recv().unwrap();
        CString::new(serde_json::to_string(&data_consumer_options).unwrap())
            .unwrap()
            .into_raw()
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
        let _ = shared.data_consumer_tx.send(data_consumer::Message::Data {
            data_consumer_id: data_consumer_id_cstr.to_string_lossy().into_owned().into(),
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
            .data_consumer_tx
            .send(data_consumer::Message::StateChanged {
                data_consumer_id: data_consumer_id_cstr.to_string_lossy().into_owned().into(),
                state: data_consumer::DataChannelState::from_str(
                    state_cstr.to_string_lossy().as_ref(),
                )
                .unwrap(),
            });
    }
}

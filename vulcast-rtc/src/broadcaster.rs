use futures::future::BoxFuture;
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
use crate::types::*;
use vulcast_rtc_sys as sys;

lazy_static! {
    static ref NATIVE_RUNTIME: Runtime = runtime::Builder::new_multi_thread().build().unwrap();
}

#[derive(Clone)]
pub struct Broadcaster {
    shared: Arc<Shared>,
}
struct Shared {
    state: Mutex<State>,
    handlers: Handlers,

    data_consumer_tx: broadcast::Sender<data_consumer::Message>,
}
// uses of sys_broadcaster are guarded by mutex, so this is safe
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
    pub fn new(handlers: Handlers) -> Self {
        super::native_init();
        let shared = Arc::new(Shared {
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
            let recv_transport_id_ptr =
                sys::broadcaster_get_recv_transport_id(self.get_sys_broadcaster());
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

    pub async fn consume_data(&self, data_producer_id: DataProducerId) -> DataConsumer {
        unsafe {
            let recv_transport_id = self.get_recv_transport_id();

            let data_consumer_options =
                (self.shared.handlers.consume_data)(recv_transport_id, data_producer_id.clone())
                    .await;

            let data_consumer = DataConsumer::new(
                self.get_sys_broadcaster(),
                data_consumer_options,
                self.shared.data_consumer_tx.clone(),
            );
            data_consumer
        }
    }

    pub fn produce_fake_media(&self) {
        unsafe {
            sys::producer_new_from_fake_audio(self.get_sys_broadcaster());
            sys::producer_new_from_fake_video(self.get_sys_broadcaster());
        }
    }

    pub fn produce_video_from_vcm_capturer(&self) {
        unsafe {
            sys::producer_new_from_vcm_capturer(self.get_sys_broadcaster());
        }
    }

    fn get_sys_broadcaster(&self) -> *mut sys::Broadcaster {
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

extern "C" fn server_rtp_capabilities(ctx: *const c_void) -> *mut i8 {
    log::trace!("server_rtp_capabilities({:?})", ctx);
    unsafe {
        let shared = &*(ctx as *const Shared);

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
}
extern "C" fn create_webrtc_transport(ctx: *const c_void) -> *mut i8 {
    log::trace!("create_webrtc_transport({:?})", ctx);
    unsafe {
        let shared = &*(ctx as *const Shared);

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
}
extern "C" fn on_rtp_capabilities(ctx: *const c_void, rtp_caps: *const c_char) {
    log::trace!("on_rtp_capabilities({:?})", ctx);
    unsafe {
        let shared = &*(ctx as *const Shared);
        let rtp_caps = CStr::from_ptr(rtp_caps).to_str().unwrap();

        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let fut = (shared.handlers.on_rtp_capabilities)(RtpCapabilities::from(
            serde_json::from_str::<serde_json::Value>(rtp_caps).unwrap(),
        ));
        NATIVE_RUNTIME.spawn(async move {
            let _ = tx.send(fut.await);
        });
        let _ = rx.recv().unwrap();
    }
}
extern "C" fn on_produce(
    ctx: *const c_void,
    transport_id: *const i8,
    kind: *const i8,
    rtp_parameters: *const i8,
) -> *mut i8 {
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
    transport_id: *const i8,
    dtls_parameters: *const i8,
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
    transport_id: *const i8,
    data_producer_id: *const i8,
) -> *mut i8 {
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
    data_consumer_id: *const i8,
    data: *const i8,
    len: u64,
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
    data_consumer_id: *const i8,
    state: *const i8,
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

use std::ptr;
use std::str::FromStr;
use std::sync::Once;
use std::sync::{Arc, Mutex};
use std::{
    ffi::{c_void, CStr, CString},
    os::raw::c_char,
};
use tokio::sync::broadcast;

use crate::data_consumer::*;
use crate::types::*;
use vulcast_rtc_sys as sys;

static NATIVE_INIT: Once = Once::new();

#[derive(Clone)]
pub struct Broadcaster {
    shared: Arc<Shared>,
}
struct Shared {
    state: Mutex<State>,
    handlers: Handlers,

    message_tx: broadcast::Sender<Message>,
}
struct State {
    sys_broadcaster: *mut sys::Broadcaster,
}
pub struct Handlers {
    pub server_rtp_capabilities: Box<dyn Fn() -> RtpCapabilitiesFinalized + Send + Sync + 'static>,
    pub create_webrtc_transport: Box<dyn Fn() -> WebRtcTransportOptions + Send + Sync + 'static>,
    pub on_rtp_capabilities: Box<dyn Fn(RtpCapabilities) + Send + Sync + 'static>,
    pub on_produce:
        Box<dyn Fn(TransportId, MediaKind, RtpParameters) -> ProducerId + Send + Sync + 'static>,
    pub on_connect_webrtc_transport:
        Box<dyn Fn(TransportId, DtlsParameters) + Send + Sync + 'static>,
    pub on_consume_data: Box<dyn Fn(TransportId, DataProducerId) + Send + Sync + 'static>,
}

impl Broadcaster {
    pub fn new(handlers: Handlers) -> Self {
        NATIVE_INIT.call_once(|| {
            let argv0 = std::env::args().next().unwrap();
            unsafe {
                let c_str = CString::new(argv0).unwrap();
                sys::init(c_str.as_ptr());
            }
        });

        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                sys_broadcaster: ptr::null_mut(),
            }),
            handlers,
            message_tx: broadcast::channel(16).0,
        });
        unsafe {
            let sys_broadcaster = sys::create_broadcaster(
                &*shared as *const _ as *const c_void,
                sys::SignalHandler {
                    server_rtp_capabilities: Some(server_rtp_capabilities),
                    on_rtp_capabilities: Some(on_rtp_capabilities),
                    on_produce: Some(on_produce),
                    on_connect_webrtc_transport: Some(on_connect_webrtc_transport),
                    on_consume_data: Some(on_consume_data),
                    create_webrtc_transport: Some(create_webrtc_transport),
                    on_message: Some(on_message),
                },
            );
            shared.set_sys_broadcaster(sys_broadcaster);
        }
        Self { shared }
    }

    pub fn consume_data(
        &self,
        data_consumer_id: DataConsumerId,
        data_producer_id: DataProducerId,
    ) -> DataConsumer {
        unsafe {
            let data_consumer_id_cstr =
                CString::new(String::from(data_consumer_id.clone())).unwrap();
            let data_producer_id_cstr = CString::new(String::from(data_producer_id)).unwrap();
            let sys_data_consumer = sys::create_data_consumer(
                self.shared.get_sys_broadcaster(),
                data_consumer_id_cstr.as_ptr(),
                data_producer_id_cstr.as_ptr(),
            );
            DataConsumer::new(
                sys_data_consumer,
                data_consumer_id,
                self.shared.message_tx.clone(),
            )
        }
    }

    // fn context(&self) -> *const c_void {
    //     &*self.shared as *const _ as *const c_void
    // }
}

impl Drop for Shared {
    fn drop(&mut self) {
        unsafe { sys::stop_broadcaster(self.get_sys_broadcaster()) }
    }
}
impl Shared {
    fn set_sys_broadcaster(&self, sys_broadcaster: *mut sys::Broadcaster) {
        let mut state = self.state.lock().unwrap();
        state.sys_broadcaster = sys_broadcaster;
    }
    fn get_sys_broadcaster(&self) -> *mut sys::Broadcaster {
        let state = self.state.lock().unwrap();
        state.sys_broadcaster
    }
}

extern "C" fn server_rtp_capabilities(ctx: *const c_void) -> *mut i8 {
    unsafe {
        let shared = &*(ctx as *const Shared);
        let server_rtp_capabilities = (shared.handlers.server_rtp_capabilities)();
        CString::new(String::from(server_rtp_capabilities))
            .unwrap()
            .into_raw()
    }
}
extern "C" fn create_webrtc_transport(ctx: *const c_void) -> *mut i8 {
    unsafe {
        let shared = &*(ctx as *const Shared);
        let webrtc_transport_options = (shared.handlers.create_webrtc_transport)();
        CString::new(String::from(webrtc_transport_options))
            .unwrap()
            .into_raw()
    }
}
extern "C" fn on_rtp_capabilities(ctx: *const c_void, rtp_caps: *const c_char) {
    unsafe {
        let shared = &*(ctx as *const Shared);
        let rtp_caps = CStr::from_ptr(rtp_caps).to_str().unwrap();
        (shared.handlers.on_rtp_capabilities)(RtpCapabilities::from(rtp_caps.to_owned()));
    }
}
extern "C" fn on_produce(
    ctx: *const c_void,
    transport_id: *const i8,
    kind: *const i8,
    rtp_parameters: *const i8,
) -> *mut i8 {
    unsafe {
        let shared = &*(ctx as *const Shared);
        let transport_id_cstr = CStr::from_ptr(transport_id);
        let kind_cstr = CStr::from_ptr(kind);
        let rtp_parameters_cstr = CStr::from_ptr(rtp_parameters);
        let producer_id = (shared.handlers.on_produce)(
            transport_id_cstr.to_string_lossy().into_owned().into(),
            MediaKind::from_str(kind_cstr.to_string_lossy().as_ref()).unwrap(),
            rtp_parameters_cstr.to_string_lossy().into_owned().into(),
        );
        CString::new(String::from(producer_id)).unwrap().into_raw()
    }
}
extern "C" fn on_connect_webrtc_transport(
    ctx: *const c_void,
    transport_id: *const i8,
    dtls_parameters: *const i8,
) {
    unsafe {
        let shared = &*(ctx as *const Shared);
        let transport_id_cstr = CStr::from_ptr(transport_id);
        let dtls_parameters_cstr = CStr::from_ptr(dtls_parameters);
        (shared.handlers.on_connect_webrtc_transport)(
            transport_id_cstr.to_string_lossy().into_owned().into(),
            dtls_parameters_cstr.to_string_lossy().into_owned().into(),
        );
    }
}
extern "C" fn on_consume_data(
    ctx: *const c_void,
    transport_id: *const i8,
    data_producer_id: *const i8,
) {
    unsafe {
        let shared = &*(ctx as *const Shared);
        let transport_id_cstr = CStr::from_ptr(transport_id);
        let data_producer_id_cstr = CStr::from_ptr(data_producer_id);
        (shared.handlers.on_consume_data)(
            transport_id_cstr.to_string_lossy().into_owned().into(),
            data_producer_id_cstr.to_string_lossy().into_owned().into(),
        );
    }
}
extern "C" fn on_message(
    ctx: *const c_void,
    data_consumer_id: *const i8,
    data: *const i8,
    len: u64,
) {
    unsafe {
        let shared = &*(ctx as *const Shared);
        let data_consumer_id_cstr = CStr::from_ptr(data_consumer_id);
        let message_data = std::slice::from_raw_parts(data as *const u8, len as usize).to_vec();
        let _ = shared.message_tx.send(Message {
            data_consumer_id: data_consumer_id_cstr.to_string_lossy().into_owned().into(),
            data: message_data,
        });
    }
}

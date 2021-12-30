use std::{
    pin::Pin,
    ptr,
    sync::{Arc, Mutex},
};

use vulcast_rtc_sys as sys;

#[derive(Clone)]
pub struct AlsaCapturer {
    _shared: Pin<Arc<Shared>>,
}
struct Shared {
    state: Mutex<State>,
}
unsafe impl Send for Shared {}
unsafe impl Sync for Shared {}
struct State {
    sys_producer: *mut sys::mediasoupclient_Producer,
}

impl AlsaCapturer {
    pub fn new(sys_broadcaster: *mut sys::Broadcaster) -> Self {
        let shared = Arc::pin(Shared {
            state: Mutex::new(State {
                sys_producer: ptr::null_mut(),
            }),
        });
        unsafe {
            let sys_producer = sys::producer_new_from_default_audio(sys_broadcaster);
            let mut state = shared.state.lock().unwrap();
            state.sys_producer = sys_producer;
        }
        AlsaCapturer { _shared: shared }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        log::trace!("producer delete {:?}", &self.sys_producer);
        unsafe {
            sys::producer_delete(self.sys_producer);
        }
    }
}

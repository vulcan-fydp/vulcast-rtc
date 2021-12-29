use std::{
    ffi::c_void,
    pin::Pin,
    ptr,
    sync::{Arc, Mutex},
};

use vulcast_rtc_sys as sys;

use crate::frame_source::FrameSource;

#[derive(Clone)]
pub struct ForeignProducer {
    _shared: Pin<Arc<Shared>>,
}
struct Shared {
    state: Mutex<State>,
    frame_source: Arc<dyn FrameSource>,
}
unsafe impl Send for Shared {}
unsafe impl Sync for Shared {}
struct State {
    sys_producer: *mut sys::mediasoupclient_Producer,
}

impl ForeignProducer {
    pub fn new(
        sys_broadcaster: *mut sys::Broadcaster,
        frame_source: Arc<dyn FrameSource>,
        width: u32,
        height: u32,
        fps: u32,
    ) -> Self {
        let shared = Arc::pin(Shared {
            state: Mutex::new(State {
                sys_producer: ptr::null_mut(),
            }),
            frame_source,
        });
        unsafe {
            let ctx = &*shared as *const _ as *mut c_void;
            let sys_producer = sys::producer_new_from_foreign(
                sys_broadcaster,
                width,
                height,
                fps,
                ctx,
                Some(frame_source_next_frame),
            );
            let mut state = shared.state.lock().unwrap();
            state.sys_producer = sys_producer;
        }
        ForeignProducer { _shared: shared }
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

extern "C" fn frame_source_next_frame(
    ctx: *const c_void,
    width: u32,
    height: u32,
    timestamp: i64,
    data: *mut u8,
) {
    unsafe {
        let shared = &*(ctx as *const Shared);
        shared.frame_source.next_frame(
            width as u32,
            height as u32,
            timestamp,
            std::slice::from_raw_parts_mut(data, (width * height * 4) as usize),
        );
    }
}

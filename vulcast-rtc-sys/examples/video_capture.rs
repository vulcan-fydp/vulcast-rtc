use std::{error::Error, ffi::CString};

use vulcast_rtc_sys as sys;

fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        let argv0 = CString::new(std::env::args().next().unwrap())?;

        sys::init(argv0.as_ptr());

        sys::set_glog_log_level(sys::GlogLogLevel_INFO);
        sys::set_mediasoup_log_level(sys::MediasoupLogLevel_LOG_DEBUG);
        sys::set_rtc_log_level(sys::RtcLogLevel_LS_WARNING);

        sys::debug_enumerate_capture_devices();
        Ok(())
    }
}

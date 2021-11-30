pub mod broadcaster;
pub mod data_consumer;
pub mod foreign_producer;
pub mod frame_source;
pub mod types;

use std::{ffi::CString, sync::Once};

use vulcast_rtc_sys as sys;

static NATIVE_INIT: Once = Once::new();

pub enum LogLevel {
    /// glog=INFO, ms=DEBUG, rtc=VERBOSE
    Spam,
    /// glog=INFO, ms=WARN, rtc=INFO
    Verbose,
    /// glog=INFO, ms=WARN, rtc=WARNING
    Debug,
    /// glog=WARNING, ms=WARN, rtc=WARNING
    Warning,
    /// glog=ERROR, ms=ERROR, rtc=ERROR
    Error,
}

fn native_init() {
    NATIVE_INIT.call_once(|| {
        log::trace!("native_init()");
        let argv0 = std::env::args().next().unwrap();
        unsafe {
            let c_str = CString::new(argv0).unwrap();
            sys::init(c_str.as_ptr());
        }
    });
}

/// Set log levels for all native modules (vulcast-rtc, mediasoup, webrtc).
pub fn set_native_log_level(level: LogLevel) {
    native_init();
    unsafe {
        match level {
            LogLevel::Spam => {
                sys::set_glog_log_level(sys::GlogLogLevel_INFO);
                sys::set_mediasoup_log_level(sys::MediasoupLogLevel_LOG_DEBUG);
                sys::set_rtc_log_level(sys::RtcLogLevel_LS_VERBOSE);
            }
            LogLevel::Verbose => {
                sys::set_glog_log_level(sys::GlogLogLevel_INFO);
                sys::set_mediasoup_log_level(sys::MediasoupLogLevel_LOG_WARN);
                sys::set_rtc_log_level(sys::RtcLogLevel_LS_INFO);
            }
            LogLevel::Debug => {
                sys::set_glog_log_level(sys::GlogLogLevel_INFO);
                sys::set_mediasoup_log_level(sys::MediasoupLogLevel_LOG_WARN);
                sys::set_rtc_log_level(sys::RtcLogLevel_LS_WARNING);
            }
            LogLevel::Warning => {
                sys::set_glog_log_level(sys::GlogLogLevel_WARNING);
                sys::set_mediasoup_log_level(sys::MediasoupLogLevel_LOG_WARN);
                sys::set_rtc_log_level(sys::RtcLogLevel_LS_WARNING);
            }
            LogLevel::Error => {
                sys::set_glog_log_level(sys::GlogLogLevel_ERROR);
                sys::set_mediasoup_log_level(sys::MediasoupLogLevel_LOG_ERROR);
                sys::set_rtc_log_level(sys::RtcLogLevel_LS_ERROR);
            }
        }
    }
}

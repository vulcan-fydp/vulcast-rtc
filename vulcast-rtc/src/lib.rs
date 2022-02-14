pub mod alsa_capturer;
pub mod broadcaster;
pub mod data_channel;
pub mod foreign_producer;
pub mod frame_source;
pub mod types;
pub mod vcm_capturer;

use std::{ffi::CString, sync::Once};

use vulcast_rtc_sys as sys;

static NATIVE_INIT: Once = Once::new();

fn native_init() {
    NATIVE_INIT.call_once(|| {
        log::trace!("native_init()");
        let argv0 = std::env::args().next().unwrap();
        unsafe {
            let c_str = CString::new(argv0).unwrap();
            sys::init(c_str.as_ptr());
            match log::max_level() {
                log::LevelFilter::Off => {}
                log::LevelFilter::Error => {
                    sys::set_glog_log_level(sys::GlogLogLevel_ERROR);
                    sys::set_mediasoup_log_level(sys::MediasoupLogLevel_LOG_ERROR);
                    sys::set_rtc_log_level(sys::RtcLogLevel_LS_ERROR);
                }
                log::LevelFilter::Warn => {
                    sys::set_glog_log_level(sys::GlogLogLevel_WARNING);
                    sys::set_mediasoup_log_level(sys::MediasoupLogLevel_LOG_WARN);
                    sys::set_rtc_log_level(sys::RtcLogLevel_LS_WARNING);
                }
                log::LevelFilter::Info => {
                    sys::set_glog_log_level(sys::GlogLogLevel_INFO);
                    sys::set_mediasoup_log_level(sys::MediasoupLogLevel_LOG_WARN);
                    sys::set_rtc_log_level(sys::RtcLogLevel_LS_WARNING);
                }
                log::LevelFilter::Debug => {
                    sys::set_glog_log_level(sys::GlogLogLevel_INFO);
                    sys::set_mediasoup_log_level(sys::MediasoupLogLevel_LOG_DEBUG);
                    sys::set_rtc_log_level(sys::RtcLogLevel_LS_INFO);
                }
                log::LevelFilter::Trace => {
                    sys::set_glog_log_level(sys::GlogLogLevel_INFO);
                    sys::set_mediasoup_log_level(sys::MediasoupLogLevel_LOG_TRACE);
                    sys::set_rtc_log_level(sys::RtcLogLevel_LS_VERBOSE);
                }
            }
        }
    });
}

pub mod broadcaster;
pub mod data_consumer;
pub mod types;

use vulcast_rtc_sys as sys;

pub enum LogLevel {
    Debug,
    Warning,
    Error,
}

/// Set log levels for all native modules (vulcast-rtc, mediasoup, webrtc). Do
/// not call before Broadcaster::new
pub fn set_native_log_level(level: LogLevel) {
    unsafe {
        match level {
            LogLevel::Debug => {
                sys::set_glog_log_level(sys::GlogLogLevel_INFO);
                sys::set_mediasoup_log_level(sys::MediasoupLogLevel_LOG_DEBUG);
                sys::set_rtc_log_level(sys::RtcLogLevel_LS_INFO);
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

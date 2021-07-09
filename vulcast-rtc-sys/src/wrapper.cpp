#include "wrapper.hpp"

#include <iostream>
#include <unordered_map>

#include <glog/logging.h>
#include <mediasoupclient.hpp>

// std::unordered_map<void*, Broadcaster> bcasters;

void hello() {
  std::cout << "charo-" << std::endl;
}

void init() {
  google::InitGoogleLogging("vulcast-rtc");
  mediasoupclient::Initialize();
  LOG(INFO) << "INIT";
}
void run(void *ctx, Signaller signaller) {}

void stop(void *ctx) {}

void set_mediasoup_log_level(MediasoupLogLevel level) {
  mediasoupclient::Logger::SetLogLevel(
      static_cast<mediasoupclient::Logger::LogLevel>(level));
}
void set_rtc_log_level(RtcLogLevel level) {
  rtc::LogMessage::LogToDebug(static_cast<rtc::LoggingSeverity>(level));
}
#include "wrapper.hpp"

#include <exception>
#include <memory>
#include <unordered_map>

#include <glog/logging.h>
#include <mediasoupclient.hpp>

#include "broadcaster.hpp"

void init(const char *argv0) {
  google::InitGoogleLogging(argv0);
  google::InstallFailureSignalHandler();
  FLAGS_logtostderr = true;
  mediasoupclient::Initialize();
  mediasoupclient::Logger::SetDefaultHandler();
}

Broadcaster *create_broadcaster(const void *ctx, SignalHandler signal_handler) {
  try {
    LOG(INFO) << "starting broadcaster";
    auto broadcaster = new Broadcaster(Signaller(ctx, signal_handler));
    broadcaster->Start();
    return broadcaster;
  } catch (const std::exception &e) {
    LOG(FATAL) << "unhandled exception: " << e.what();
  }
  return nullptr;
}

void stop_broadcaster(Broadcaster *broadcaster) {
  LOG(INFO) << "stopping broadcaster";
  delete broadcaster;
}

mediasoupclient::DataConsumer *
create_data_consumer(Broadcaster *b, const char *data_consumer_id,
                     const char *data_producer_id) {
  return b->CreateDataConsumer(data_consumer_id, data_producer_id);
}
void stop_data_consumer(mediasoupclient::DataConsumer *consumer) {
  consumer->Close();
}

void set_mediasoup_log_level(MediasoupLogLevel level) {
  mediasoupclient::Logger::SetLogLevel(
      static_cast<mediasoupclient::Logger::LogLevel>(level));
}
void set_rtc_log_level(RtcLogLevel level) {
  rtc::LogMessage::LogToDebug(static_cast<rtc::LoggingSeverity>(level));
}
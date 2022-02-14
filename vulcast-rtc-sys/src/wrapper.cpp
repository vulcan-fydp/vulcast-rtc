#include "wrapper.hpp"

#include <memory>
#include <unordered_map>

#include <common_video/libyuv/include/webrtc_libyuv.h>
#include <glog/logging.h>
#include <mediasoupclient.hpp>
#include <modules/video_capture/video_capture.h>
#include <modules/video_capture/video_capture_factory.h>

#include "broadcaster.hpp"
#include "media_stream_track_factory.hpp"

namespace {
[[nodiscard]] char *cpp_marshal_str(const std::string &str) {
  char *buf = new char[str.length() + 1];
  std::strcpy(buf, str.c_str());
  return buf;
}
} // namespace

void init(const char *argv0) {
  google::InitGoogleLogging(argv0);
  FLAGS_logtostderr = true;
  FLAGS_minloglevel = 3;
  mediasoupclient::Initialize();
  mediasoupclient::Logger::SetDefaultHandler();
}

Broadcaster *broadcaster_new(const void *ctx, SignalHandler signal_handler) {
  LOG(INFO) << "broadcaster_new(" << std::hex << ctx << ")";
  auto broadcaster = new Broadcaster(Signaller(ctx, signal_handler));
  broadcaster->Start();
  return broadcaster;
}
void broadcaster_delete(Broadcaster *broadcaster) {
  LOG(INFO) << "broadcaster_delete(" << std::hex << broadcaster << ")";
  delete broadcaster;
}
char *broadcaster_marshal_recv_transport_id(Broadcaster *b) {
  return cpp_marshal_str(b->GetRecvTransportId());
}

mediasoupclient::DataConsumer *
data_consumer_new(Broadcaster *b, const char *data_consumer_id,
                  const char *data_producer_id,
                  const char *sctp_stream_parameters) {
  LOG(INFO) << "data_consumer_new(" << std::hex << b << "," << data_consumer_id
            << "," << data_producer_id << "," << sctp_stream_parameters << ")";
  return b->ConsumeData(data_consumer_id, data_producer_id,
                        nlohmann::json::parse(sctp_stream_parameters));
}
void data_consumer_delete(mediasoupclient::DataConsumer *consumer) {
  LOG(INFO) << "data_consumer_delete(" << consumer->GetId() << ")";
  consumer->Close();
}

mediasoupclient::Producer *producer_new_from_default_audio(Broadcaster *b) {
  LOG(INFO) << "producer_new_from_default_audio(" << std::hex << b << ")";
  CHECK(b->CanProduceAudio());
  auto audio_track = CreateAudioTrack();
  nlohmann::json codec_options = {{"opusStereo", true}, {"opusDtx", true}};
  auto producer = b->Produce(audio_track, nullptr, codec_options);
  CHECK(producer != nullptr);
  return producer;
}
mediasoupclient::Producer *producer_new_from_fake_video(Broadcaster *b) {
  LOG(INFO) << "producer_new_from_fake_video(" << std::hex << b << ")";
  CHECK(b->CanProduceVideo());
  auto video_track = CreateSquaresVideoTrack();
  auto producer = b->Produce(video_track);
  CHECK(producer != nullptr);
  return producer;
}
mediasoupclient::Producer *
producer_new_from_vcm_capturer(Broadcaster *b, int device_idx, uint32_t width,
                               uint32_t height, uint32_t fps, int video_type) {
  LOG(INFO) << "producer_new_from_vcm_capturer(" << std::hex << b << ")";
  auto video_track =
      CreateVcmCapturerVideoTrack(device_idx, width, height, fps,
                                  static_cast<webrtc::VideoType>(video_type));
  auto producer = b->Produce(video_track);
  CHECK(producer != nullptr);
  return producer;
}
mediasoupclient::Producer *
producer_new_from_foreign(Broadcaster *b, uint32_t width, uint32_t height,
                          uint32_t fps, void *ctx, frame_callback_t callback) {
  LOG(INFO) << "producer_new_from_foreign(" << std::hex << b << ")";
  auto video_track = CreateForeignVideoTrack(width, height, fps, ctx, callback);
  auto producer = b->Produce(video_track);
  CHECK(producer != nullptr);
  return producer;
}
void producer_delete(mediasoupclient::Producer *producer) {
  LOG(INFO) << "producer_delete(" << std::hex << producer << ")";
  CHECK(producer != nullptr);
  producer->Close();
}
mediasoupclient::DataProducer *data_producer_new(Broadcaster *b) {
  LOG(INFO) << "data_producer_new(" << std::hex << b << ")";
  auto data_producer = b->ProduceData();
  CHECK(data_producer != nullptr);
  return data_producer;
}
char *data_producer_marshal_id(mediasoupclient::DataProducer *data_producer) {
  return cpp_marshal_str(data_producer->GetId());
}
void data_producer_send(mediasoupclient::DataProducer *data_producer,
                        const uint8_t *data, size_t len) {
  webrtc::DataBuffer data_buffer(rtc::CopyOnWriteBuffer(data, len), true);
  data_producer->Send(data_buffer);
}
void data_producer_delete(mediasoupclient::DataProducer *data_producer) {
  LOG(INFO) << "data_producer_delete(" << std::hex << data_producer << ")";
  CHECK(data_producer != nullptr);
  data_producer->Close();
}

void debug_enumerate_capture_devices() {
  LOG(INFO) << "debug_enumerate_capture_devices()";
  std::unique_ptr<webrtc::VideoCaptureModule::DeviceInfo> info(
      webrtc::VideoCaptureFactory::CreateDeviceInfo());
  CHECK(info);
  const size_t device_count = info->NumberOfDevices();
  for (size_t i = 0; i < device_count; ++i) {
    char device_name[256];
    char unique_name[256];
    CHECK(info->GetDeviceName(i, device_name, sizeof(device_name), unique_name,
                              sizeof(unique_name)) == 0);
    LOG(INFO) << i << ": device_name=" << device_name
              << " unique_name=" << unique_name;
    webrtc::VideoCaptureCapability video_caps;
    const int cap_count = info->NumberOfCapabilities(unique_name);
    for (int j = 0; j < cap_count; ++j) {
      CHECK(info->GetCapability(unique_name, j, video_caps) == 0);
      int fourcc = ConvertVideoType(video_caps.videoType);
      char fourcc_str[5] = {0};
      std::memcpy(fourcc_str, &fourcc, 4);
      LOG(INFO) << "\t" << j << ": fourcc=" << fourcc_str << " "
                << video_caps.width << "x" << video_caps.height << "@"
                << video_caps.maxFPS << "fps itl=" << video_caps.interlaced;
    }
  }
}

void set_glog_log_level(GlogLogLevel level) { FLAGS_minloglevel = level; }
void set_mediasoup_log_level(MediasoupLogLevel level) {
  mediasoupclient::Logger::SetLogLevel(
      static_cast<mediasoupclient::Logger::LogLevel>(level));
}
void set_rtc_log_level(RtcLogLevel level) {
  rtc::LogMessage::LogToDebug(static_cast<rtc::LoggingSeverity>(level));
}

void cpp_unmarshal_str(char *str) { delete[] str; }
#include <api/scoped_refptr.h>
#include <api/task_queue/default_task_queue_factory.h>
#include <iostream>

#include "api/audio_codecs/builtin_audio_decoder_factory.h"
#include "api/audio_codecs/builtin_audio_encoder_factory.h"
#include "api/create_peerconnection_factory.h"
#include "api/video_codecs/builtin_video_decoder_factory.h"
#include "api/video_codecs/builtin_video_encoder_factory.h"
#include "media_stream_track_factory.hpp"
#include "modules/video_capture/video_capture.h"
#include "modules/video_capture/video_capture_factory.h"
#include "pc/test/fake_audio_capture_module.h"
#include "pc/test/fake_periodic_video_track_source.h"
#include "pc/test/frame_generator_capturer_video_track_source.h"
#include "system_wrappers/include/clock.h"
#include "test/vcm_capturer.h"

#include <glog/logging.h>

#ifdef VULCAST_RTC_RPI
#include "raspi_decoder.h"
#include "raspi_encoder.h"
#endif

namespace {

static rtc::scoped_refptr<webrtc::PeerConnectionFactoryInterface>
CreatePeerConnectionFactory() {
  auto networkThread = rtc::Thread::CreateWithSocketServer().release();
  auto signalingThread = rtc::Thread::Create().release();
  auto workerThread = rtc::Thread::Create().release();

  networkThread->SetName("network_thread", nullptr);
  signalingThread->SetName("signaling_thread", nullptr);
  workerThread->SetName("worker_thread", nullptr);

  if (!networkThread->Start() || !signalingThread->Start() ||
      !workerThread->Start()) {
    LOG(FATAL) << "thread start errored";
  }

  auto fakeAudioCaptureModule = FakeAudioCaptureModule::Create();
  if (!fakeAudioCaptureModule) {
    LOG(FATAL) << "audio capture module creation errored";
  }

  return webrtc::CreatePeerConnectionFactory(
      networkThread, workerThread, signalingThread, fakeAudioCaptureModule,
      webrtc::CreateBuiltinAudioEncoderFactory(),
      webrtc::CreateBuiltinAudioDecoderFactory(),
      // #ifdef VULCAST_RTC_RPI
      //       webrtc::CreateRaspiVideoEncoderFactory(),
      //       webrtc::CreateRaspiVideoDecoderFactory(),
      // #else
      webrtc::CreateBuiltinVideoEncoderFactory(),
      webrtc::CreateBuiltinVideoDecoderFactory(),
      // #endif
      nullptr /*audio_mixer*/, nullptr /*audio_processing*/);
}
} // namespace

rtc::scoped_refptr<webrtc::PeerConnectionFactoryInterface>
GetPeerConnectionFactory() {
  static rtc::scoped_refptr<webrtc::PeerConnectionFactoryInterface>
      peer_connection_factory = CreatePeerConnectionFactory();
  DCHECK(peer_connection_factory);
  return peer_connection_factory;
}

// Audio track creation.
rtc::scoped_refptr<webrtc::AudioTrackInterface> CreateAudioTrack() {
  auto factory = GetPeerConnectionFactory();

  cricket::AudioOptions options;
  options.highpass_filter = false;

  rtc::scoped_refptr<webrtc::AudioSourceInterface> source =
      factory->CreateAudioSource(options);

  return factory->CreateAudioTrack(rtc::CreateRandomUuid(), source);
}

// Video track creation.
rtc::scoped_refptr<webrtc::VideoTrackInterface> CreateVideoTrack() {
  auto factory = GetPeerConnectionFactory();

  auto *videoTrackSource =
      new rtc::RefCountedObject<webrtc::FakePeriodicVideoTrackSource>(
          false /* remote */);

  return factory->CreateVideoTrack(rtc::CreateRandomUuid(), videoTrackSource);
}

rtc::scoped_refptr<webrtc::VideoTrackInterface> CreateSquaresVideoTrack() {
  auto factory = GetPeerConnectionFactory();

  LOG(INFO) << "getting frame generator";
  auto *videoTrackSource =
      new rtc::RefCountedObject<webrtc::FrameGeneratorCapturerVideoTrackSource>(
          webrtc::FrameGeneratorCapturerVideoTrackSource::Config(),
          webrtc::Clock::GetRealTimeClock(), false);
  videoTrackSource->Start();

  LOG(INFO) << "[INFO] creating video track";
  return factory->CreateVideoTrack(rtc::CreateRandomUuid(), videoTrackSource);
}

class CapturerTrackSource : public webrtc::VideoTrackSource {
public:
  static rtc::scoped_refptr<CapturerTrackSource>
  Create(int device_idx, size_t width, size_t height, size_t fps) {
    std::unique_ptr<webrtc::test::VcmCapturer> capturer;
    std::unique_ptr<webrtc::VideoCaptureModule::DeviceInfo> info(
        webrtc::VideoCaptureFactory::CreateDeviceInfo());
    if (!info) {
      return nullptr;
    }
    int num_devices = info->NumberOfDevices();
    if (device_idx > 0) {
      CHECK(device_idx < num_devices);
      capturer = absl::WrapUnique(
          webrtc::test::VcmCapturer::Create(width, height, fps, device_idx));
      if (capturer) {
        return new rtc::RefCountedObject<CapturerTrackSource>(
            std::move(capturer));
      }
    } else {
      for (int i = 0; i < num_devices; ++i) {
        capturer = absl::WrapUnique(
            webrtc::test::VcmCapturer::Create(width, height, fps, i));
        if (capturer) {
          return new rtc::RefCountedObject<CapturerTrackSource>(
              std::move(capturer));
        }
      }
    }

    return nullptr;
  }

protected:
  explicit CapturerTrackSource(
      std::unique_ptr<webrtc::test::VcmCapturer> capturer)
      : VideoTrackSource(/*remote=*/false), capturer_(std::move(capturer)) {}

private:
  rtc::VideoSourceInterface<webrtc::VideoFrame> *source() override {
    return capturer_.get();
  }
  std::unique_ptr<webrtc::test::VcmCapturer> capturer_;
};

rtc::scoped_refptr<webrtc::VideoTrackInterface>
CreateVcmCapturerVideoTrack(int device_idx, size_t width, size_t height,
                            size_t fps) {
  auto factory = GetPeerConnectionFactory();

  rtc::scoped_refptr<CapturerTrackSource> video_device =
      CapturerTrackSource::Create(device_idx, width, height, fps);
  CHECK(video_device);
  return factory->CreateVideoTrack(rtc::CreateRandomUuid(), video_device);
}

rtc::scoped_refptr<webrtc::VideoTrackInterface>
CreateForeignVideoTrack(size_t width, size_t height, size_t fps, void *ctx,
                        frame_callback_t callback) {
  auto factory = GetPeerConnectionFactory();

  auto task_queue_factory = webrtc::CreateDefaultTaskQueueFactory();
  auto video_capturer = std::make_unique<webrtc::test::FrameGeneratorCapturer>(
      webrtc::Clock::GetRealTimeClock(),
      std::make_unique<ForeignFrameGenerator>(
          width, height, webrtc::Clock::GetRealTimeClock(), ctx, callback),
      fps, *task_queue_factory);
  video_capturer->Init();

  auto *videoTrackSource =
      new rtc::RefCountedObject<webrtc::FrameGeneratorCapturerVideoTrackSource>(
          std::move(video_capturer), true);
  videoTrackSource->Start();

  return factory->CreateVideoTrack(rtc::CreateRandomUuid(), videoTrackSource);
}
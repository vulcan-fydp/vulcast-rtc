#pragma once

#include <memory>
#include <string>
#include <vector>

#include <api/scoped_refptr.h>
#include <api/test/frame_generator_interface.h>
#include <api/video/i420_buffer.h>
#include <api/video/video_frame.h>
#include <api/video/video_frame_buffer.h>
#include <api/video/video_source_interface.h>
#include <common_video/include/i420_buffer_pool.h>
#include <rtc_base/critical_section.h>
#include <rtc_base/random.h>
#include <system_wrappers/include/clock.h>

#include "wrapper.hpp"

class ForeignFrameGenerator : public webrtc::test::FrameGeneratorInterface {
public:
  ForeignFrameGenerator(int width, int height, webrtc::Clock *clock, void *ctx,
                        frame_callback_t callback);

  void ChangeResolution(size_t width, size_t height) override;
  VideoFrameData NextFrame() override;

private:
  rtc::scoped_refptr<webrtc::I420Buffer> CreateI420Buffer(int width,
                                                          int height);
  size_t rgba_stride() const { return width_ * 4; }

  rtc::CriticalSection crit_;
  int width_ RTC_GUARDED_BY(&crit_);
  int height_ RTC_GUARDED_BY(&crit_);

  webrtc::Clock *const clock_;
  void *const ctx_;
  const frame_callback_t callback_;

  std::vector<uint8_t> rgba_buffer_ RTC_GUARDED_BY(&crit_);
  webrtc::I420BufferPool buffer_pool_ RTC_GUARDED_BY(&crit_);
};
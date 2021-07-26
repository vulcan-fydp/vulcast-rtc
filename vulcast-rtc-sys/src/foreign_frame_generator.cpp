#include "foreign_frame_generator.hpp"

#include "third_party/libyuv/include/libyuv/convert.h"

#include "glog/logging.h"

using namespace webrtc::test;

ForeignFrameGenerator::ForeignFrameGenerator(int width, int height,
                                             webrtc::Clock *clock, void *ctx,
                                             frame_callback_t callback)
    : clock_(clock), ctx_(ctx), callback_(callback) {
  ChangeResolution(width, height);
}

void ForeignFrameGenerator::ChangeResolution(size_t width, size_t height) {
  rtc::CritScope lock(&crit_);
  width_ = static_cast<int>(width);
  height_ = static_cast<int>(height);
  CHECK(width_ > 0);
  CHECK(height_ > 0);
  rgba_buffer_.resize(height_ * rgba_stride());
}

FrameGeneratorInterface::VideoFrameData ForeignFrameGenerator::NextFrame() {
  rtc::CritScope lock(&crit_);
  auto buffer = CreateI420Buffer(width_, height_);
  callback_(ctx_, width_, height_, clock_->TimeInMicroseconds(),
            rgba_buffer_.data());

  libyuv::ARGBToI420(rgba_buffer_.data(), rgba_stride(), buffer->MutableDataY(),
                     buffer->StrideY(), buffer->MutableDataU(),
                     buffer->StrideU(), buffer->MutableDataV(),
                     buffer->StrideV(), width_, height_);
  return VideoFrameData(buffer, absl::nullopt);
}

rtc::scoped_refptr<webrtc::I420Buffer>
ForeignFrameGenerator::CreateI420Buffer(int width, int height) {
  auto buffer = buffer_pool_.CreateBuffer(width, height);
  memset(buffer->MutableDataY(), 127, height * buffer->StrideY());
  memset(buffer->MutableDataU(), 127,
         buffer->ChromaHeight() * buffer->StrideU());
  memset(buffer->MutableDataV(), 127,
         buffer->ChromaHeight() * buffer->StrideV());
  return buffer;
}
#ifndef MSC_TEST_MEDIA_STREAM_TRACK_FACTORY_HPP
#define MSC_TEST_MEDIA_STREAM_TRACK_FACTORY_HPP

#include "api/media_stream_interface.h"

#include "foreign_frame_generator.hpp"

rtc::scoped_refptr<webrtc::AudioTrackInterface> createAudioTrack();

rtc::scoped_refptr<webrtc::VideoTrackInterface> createVideoTrack();

rtc::scoped_refptr<webrtc::VideoTrackInterface> createSquaresVideoTrack();

rtc::scoped_refptr<webrtc::VideoTrackInterface> createVcmCapturerVideoTrack();

rtc::scoped_refptr<webrtc::VideoTrackInterface>
createForeignVideoTrack(size_t width, size_t height, size_t fps, void *ctx,
                        frame_callback_t callback);

#endif

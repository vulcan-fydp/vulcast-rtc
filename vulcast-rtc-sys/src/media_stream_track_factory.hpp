#pragma once

#include <api/media_stream_interface.h>
#include <api/peer_connection_interface.h>

#include "foreign_frame_generator.hpp"

rtc::scoped_refptr<webrtc::PeerConnectionFactoryInterface>
GetPeerConnectionFactory();

rtc::scoped_refptr<webrtc::AudioTrackInterface> CreateAudioTrack();
rtc::scoped_refptr<webrtc::VideoTrackInterface> CreateVideoTrack();
rtc::scoped_refptr<webrtc::VideoTrackInterface> CreateSquaresVideoTrack();
rtc::scoped_refptr<webrtc::VideoTrackInterface>
CreateVcmCapturerVideoTrack(int device_idx, size_t width, size_t height, size_t fps);
rtc::scoped_refptr<webrtc::VideoTrackInterface>
CreateForeignVideoTrack(size_t width, size_t height, size_t fps, void *ctx,
                        frame_callback_t callback);

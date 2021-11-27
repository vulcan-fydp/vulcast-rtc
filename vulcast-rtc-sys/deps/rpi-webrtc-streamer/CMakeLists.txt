add_library(rpi-webrtc-streamer STATIC)

target_sources(rpi-webrtc-streamer PRIVATE 
	src/raspi_encoder.h
	src/raspi_encoder.cc
	src/raspi_encoder_impl.h
	src/raspi_encoder_impl.cc
	src/mmal_wrapper.h
	src/mmal_wrapper.cc
	src/config_media.h
	src/config_media.cc
	src/compat/optionsfile.h
	src/compat/optionsfile.cc
	src/wstreamer_types.h
	src/wstreamer_types.cc
	src/def/config_media.def
	src/frame_queue.h
	src/frame_queue.cc
	src/mmal_video.h
	src/mmal_video.c
	src/mmal_common.h
	src/raspicamcontrol.h
	src/raspicamcontrol.c
	src/raspicli.h
	src/raspicli.c
	src/raspipreview.h
	src/raspipreview.c
)
target_include_directories(rpi-webrtc-streamer SYSTEM PUBLIC 
	${CMAKE_CURRENT_SOURCE_DIR}/src
)
target_compile_options(rpi-webrtc-streamer PRIVATE "-Wall" "-Wextra")
target_link_libraries(rpi-webrtc-streamer PUBLIC webrtc rpi-firmware)
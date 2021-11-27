add_library(rpi-webrtc-streamer STATIC)

target_sources(rpi-webrtc-streamer PRIVATE 
	src/raspi_decoder.h
	src/raspi_decoder.cc
	src/raspi_decoder_dummy.h
	src/raspi_decoder_dummy.cc
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
	src/mmal_util.c
	src/mmal_video_reset.c
	src/mmal_common.h
	src/raspicamcontrol.h
	src/raspicamcontrol.c
	src/raspicli.h
	src/raspicli.c
	src/raspipreview.h
	src/raspipreview.c
	src/raspi_quality_config.h
	src/raspi_quality_config.cc
	src/utils.h
	src/utils.cc
)
target_include_directories(rpi-webrtc-streamer SYSTEM PUBLIC 
	${CMAKE_CURRENT_SOURCE_DIR}/src
)
target_compile_options(rpi-webrtc-streamer PRIVATE "-Wall" "-Wextra")
target_compile_definitions(rpi-webrtc-streamer PUBLIC
	-D__RWS_VERSION__="abb8b04ceb58c65e-dirty"
	-D__WEBRTC_VERSION__="m94.4606"
	-DINSTALL_DIR="amogus")
target_link_libraries(rpi-webrtc-streamer PUBLIC webrtc rpi-firmware)

INSTALL(TARGETS rpi-webrtc-streamer DESTINATION lib)
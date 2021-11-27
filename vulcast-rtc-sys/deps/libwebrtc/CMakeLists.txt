# libwebrtc
add_library(webrtc INTERFACE)
set(LIBWEBRTC_STATIC ${LIBWEBRTC_BINARY_PATH}/libwebrtc${CMAKE_STATIC_LIBRARY_SUFFIX})
target_include_directories(webrtc SYSTEM INTERFACE ${LIBWEBRTC_INCLUDE_PATH})
target_include_directories(webrtc SYSTEM INTERFACE ${LIBWEBRTC_INCLUDE_PATH}/third_party/libyuv/include)
target_link_libraries(webrtc INTERFACE ${LIBWEBRTC_STATIC} pthread dl webrtcextra)
target_link_libraries(${PROJECT_NAME} PUBLIC webrtc)

# test
add_library(webrtcextra STATIC)

target_sources(webrtcextra PRIVATE 
	src/api/test/create_frame_generator.cc
	src/media/base/fake_frame_source.cc
	src/pc/test/fake_audio_capture_module.cc
	src/rtc_base/task_queue_for_test.cc
	src/test/frame_generator.cc
	src/test/frame_generator_capturer.cc
	src/test/frame_utils.cc
	src/test/test_video_capturer.cc
	src/test/testsupport/ivf_video_frame_generator.cc
	src/test/testsupport/file_utils.cc
	src/test/testsupport/file_utils_override.cc
	src/test/vcm_capturer.cc
)

target_include_directories(webrtcextra SYSTEM PUBLIC
	${LIBWEBRTC_INCLUDE_PATH}
	${LIBWEBRTC_INCLUDE_PATH}/third_party/abseil-cpp
	${LIBWEBRTC_INCLUDE_PATH}/third_party/jsoncpp/source/include
)

target_compile_definitions(webrtcextra PUBLIC
	$<$<NOT:$<PLATFORM_ID:Windows>>:WEBRTC_POSIX>
	$<$<PLATFORM_ID:Windows>:WEBRTC_WIN>
	$<$<PLATFORM_ID:Darwin>:WEBRTC_MAC>
)

INSTALL(TARGETS webrtcextra DESTINATION lib)

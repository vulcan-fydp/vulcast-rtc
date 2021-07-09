add_library(webrtc_broadcaster STATIC ${PROJECT_SOURCES} ${PROJECT_HEADERS})

target_sources(webrtc_broadcaster PRIVATE 
	api/test/create_frame_generator.cc
	media/base/fake_frame_source.cc
	pc/test/fake_audio_capture_module.cc
	rtc_base/task_queue_for_test.cc
	test/frame_generator.cc
	test/frame_generator_capturer.cc
	test/frame_utils.cc
	test/test_video_capturer.cc
	test/testsupport/ivf_video_frame_generator.cc
	test/testsupport/file_utils.cc
	test/testsupport/file_utils_override.cc
)

target_include_directories(webrtc_broadcaster SYSTEM PUBLIC ${PROJECT_SOURCE_DIR}/deps/libwebrtc)
target_include_directories(webrtc_broadcaster SYSTEM PUBLIC
	"${LIBWEBRTC_INCLUDE_PATH}"
	"${LIBWEBRTC_INCLUDE_PATH}/third_party/abseil-cpp"
)

target_compile_definitions(webrtc_broadcaster PUBLIC
	$<$<NOT:$<PLATFORM_ID:Windows>>:WEBRTC_POSIX>
	$<$<PLATFORM_ID:Windows>:WEBRTC_WIN>
	$<$<PLATFORM_ID:Darwin>:WEBRTC_MAC>
)


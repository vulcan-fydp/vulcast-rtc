target_sources(${PROJECT_NAME} PRIVATE 
	broadcaster.cpp 
	broadcaster.hpp 
	media_stream_track_factory.cpp 
	media_stream_track_factory.hpp
	wrapper.hpp
	wrapper.cpp
	ffi.hpp
	signaller.hpp
	signaller.cpp
	foreign_frame_generator.hpp
	foreign_frame_generator.cpp
)
target_include_directories(${PROJECT_NAME} PUBLIC 
	${CMAKE_CURRENT_SOURCE_DIR}
)
if(${TARGET} STREQUAL "armv7-unknown-linux-gnueabihf")
	target_compile_definitions(${PROJECT_NAME} PRIVATE VULCAST_RTC_RPI)
endif()
target_compile_options(${PROJECT_NAME} PRIVATE "-Wall" "-Wextra")
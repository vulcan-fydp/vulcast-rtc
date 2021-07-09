target_sources(${PROJECT_NAME} PRIVATE 
	broadcaster.cpp 
	broadcaster.hpp 
	main.cpp 
	media_stream_track_factory.cpp 
	media_stream_track_factory.hpp
	wrapper.hpp
	wrapper.cpp
)
target_include_directories(${PROJECT_NAME} PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})
target_compile_options(${PROJECT_NAME} PRIVATE "-Wall" "-Wextra")
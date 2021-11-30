cmake_minimum_required(VERSION 3.16)

project(vulcast-rtc)

set(CMAKE_CXX_STANDARD 14)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_CXX_FLAGS_DEBUG "${CMAKE_CXX_FLAGS_DEBUG} -O0 -ggdb")

message("\n=========== vulcast-rtc Build Configuration ===========\n")
message(STATUS "TARGET                 : " ${TARGET})
message(STATUS "LIBWEBRTC_INCLUDE_PATH : " ${LIBWEBRTC_INCLUDE_PATH})
message(STATUS "LIBWEBRTC_BINARY_PATH  : " ${LIBWEBRTC_BINARY_PATH})
message("")

add_library(${PROJECT_NAME} STATIC ${PROJECT_SOURCES} ${PROJECT_HEADERS})
add_subdirectory(src)
add_subdirectory(deps)

install(TARGETS vulcast-rtc DESTINATION lib)

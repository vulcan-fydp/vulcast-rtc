include(FetchContent)

# libwebrtc
add_subdirectory(libwebrtc)

# mediasoup
FetchContent_Declare(
  mediasoupclient
  GIT_REPOSITORY https://github.com/versatica/libmediasoupclient.git
  GIT_TAG v3
)
FetchContent_MakeAvailable(mediasoupclient)
target_include_directories(${PROJECT_NAME} SYSTEM PUBLIC ${mediasoupclient_SOURCE_DIR}/include)
target_link_libraries(${PROJECT_NAME} PUBLIC mediasoupclient)

# glog
set(WITH_GFLAGS OFF CACHE INTERNAL "")
set(WITH_UNWIND OFF CACHE INTERNAL "")
set(WITH_GTEST OFF CACHE INTERNAL "")
set(BUILD_SHARED_LIBS OFF CACHE INTERNAL "")
FetchContent_Declare(
  glog
  GIT_REPOSITORY https://github.com/google/glog.git
  GIT_TAG v0.5.0
)
FetchContent_MakeAvailable(glog)
target_include_directories(${PROJECT_NAME} SYSTEM PUBLIC ${glog_SOURCE_DIR}/src)
target_link_libraries(${PROJECT_NAME} PUBLIC glog::glog)

# rpi firmware
add_subdirectory(rpi-firmware)
target_link_libraries(${PROJECT_NAME} PUBLIC rpi-firmware)
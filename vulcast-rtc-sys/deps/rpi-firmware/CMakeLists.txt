add_library(rpi-firmware INTERFACE)
target_include_directories(rpi-firmware SYSTEM INTERFACE opt/vc/include/)
target_compile_definitions(rpi-firmware INTERFACE
	-D__STDC_CONSTANT_MACROS -D__STDC_LIMIT_MACROS -DTARGET_POSIX 
	-D_LINUX -DPIC -D_REENTRANT -D_LARGEFILE64_SOURCE 
	-D_FILE_OFFSET_BITS=64 -DHAVE_LIBOPENMAX=2 
	-DOMX -DOMX_SKIP64BIT -DUSE_EXTERNAL_OMX 
	-DHAVE_LIBBCM_HOST -DUSE_EXTERNAL_LIBBCM_HOST -DUSE_VCHIQ_ARM
)
# link_directories(${CMAKE_CURRENT_SOURCE_DIR}/opt/vc/lib)
# target_link_libraries(rpi-firmware INTERFACE mmal_core mmal mmal_util bcm_host mmal_vc_client mmal_components)
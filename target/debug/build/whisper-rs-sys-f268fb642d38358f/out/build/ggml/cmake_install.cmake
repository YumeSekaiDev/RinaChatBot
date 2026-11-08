# Install script for directory: /mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/whisper.cpp/ggml

# Set the install prefix
if(NOT DEFINED CMAKE_INSTALL_PREFIX)
  set(CMAKE_INSTALL_PREFIX "/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out")
endif()
string(REGEX REPLACE "/$" "" CMAKE_INSTALL_PREFIX "${CMAKE_INSTALL_PREFIX}")

# Set the install configuration name.
if(NOT DEFINED CMAKE_INSTALL_CONFIG_NAME)
  if(BUILD_TYPE)
    string(REGEX REPLACE "^[^A-Za-z0-9_]+" ""
           CMAKE_INSTALL_CONFIG_NAME "${BUILD_TYPE}")
  else()
    set(CMAKE_INSTALL_CONFIG_NAME "RelWithDebInfo")
  endif()
  message(STATUS "Install configuration: \"${CMAKE_INSTALL_CONFIG_NAME}\"")
endif()

# Set the component getting installed.
if(NOT CMAKE_INSTALL_COMPONENT)
  if(COMPONENT)
    message(STATUS "Install component: \"${COMPONENT}\"")
    set(CMAKE_INSTALL_COMPONENT "${COMPONENT}")
  else()
    set(CMAKE_INSTALL_COMPONENT)
  endif()
endif()

# Install shared libraries without execute permission?
if(NOT DEFINED CMAKE_INSTALL_SO_NO_EXE)
  set(CMAKE_INSTALL_SO_NO_EXE "0")
endif()

# Is this installation the result of a crosscompile?
if(NOT DEFINED CMAKE_CROSSCOMPILING)
  set(CMAKE_CROSSCOMPILING "FALSE")
endif()

# Set path to fallback-tool for dependency-resolution.
if(NOT DEFINED CMAKE_OBJDUMP)
  set(CMAKE_OBJDUMP "/usr/bin/objdump")
endif()

if(NOT CMAKE_INSTALL_LOCAL_ONLY)
  # Include the install script for the subdirectory.
  include("/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/build/ggml/src/cmake_install.cmake")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib64" TYPE STATIC_LIBRARY FILES "/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/build/ggml/src/libggml.a")
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/include" TYPE FILE FILES
    "/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/whisper.cpp/ggml/include/ggml.h"
    "/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/whisper.cpp/ggml/include/ggml-cpu.h"
    "/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/whisper.cpp/ggml/include/ggml-alloc.h"
    "/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/whisper.cpp/ggml/include/ggml-backend.h"
    "/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/whisper.cpp/ggml/include/ggml-blas.h"
    "/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/whisper.cpp/ggml/include/ggml-cann.h"
    "/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/whisper.cpp/ggml/include/ggml-cuda.h"
    "/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/whisper.cpp/ggml/include/ggml-kompute.h"
    "/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/whisper.cpp/ggml/include/ggml-opt.h"
    "/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/whisper.cpp/ggml/include/ggml-metal.h"
    "/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/whisper.cpp/ggml/include/ggml-rpc.h"
    "/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/whisper.cpp/ggml/include/ggml-sycl.h"
    "/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/whisper.cpp/ggml/include/ggml-vulkan.h"
    )
endif()

if(CMAKE_INSTALL_COMPONENT STREQUAL "Unspecified" OR NOT CMAKE_INSTALL_COMPONENT)
  file(INSTALL DESTINATION "${CMAKE_INSTALL_PREFIX}/lib64" TYPE STATIC_LIBRARY FILES "/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/build/ggml/src/libggml-base.a")
endif()

string(REPLACE ";" "\n" CMAKE_INSTALL_MANIFEST_CONTENT
       "${CMAKE_INSTALL_MANIFEST_FILES}")
if(CMAKE_INSTALL_LOCAL_ONLY)
  file(WRITE "/mnt/UNTV News/Game Dev/VoiceChatRina/target/debug/build/whisper-rs-sys-f268fb642d38358f/out/build/ggml/install_local_manifest.txt"
     "${CMAKE_INSTALL_MANIFEST_CONTENT}")
endif()

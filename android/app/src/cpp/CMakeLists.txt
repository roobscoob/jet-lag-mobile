cmake_minimum_required(VERSION 3.22)

set(ANDROID_WEAK_API_DEFS ON)

project(CustomLayer CXX)

add_library(
  custom-layer-shim SHARED customLayer.cpp
)
target_link_options(custom-layer-shim PRIVATE)
find_library(log-lib log)
target_link_libraries(custom-layer-shim ${log-lib})

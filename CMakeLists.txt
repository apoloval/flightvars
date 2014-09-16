cmake_minimum_required(VERSION 2.8)

project(FlightVars)

set(CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/cmake/modules/" ${CMAKE_MODULE_PATH})

set(Boost_USE_STATIC_LIBS ON)
# If MSVC runtime static linking is used, we must use the appropriate Boost
# version of the library.
if("${MSVC_RUNTIME_LINK}" STREQUAL "static")
   set(Boost_USE_STATIC_RUNTIME ON)
endif()

find_package(Boost COMPONENTS
   chrono
   date_time
   filesystem
   log
   log_setup
   regex
   system
   thread
   unit_test_framework
REQUIRED)

message(STATUS "Boost libraries are at ${Boost_LIBRARY_DIR}")

# Add the subdirs
add_subdirectory(common)

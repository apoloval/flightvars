cmake_minimum_required(VERSION 2.8)

project(FlightVars-Common)

set(flightvars_HEADERS
   include/flightvars/concurrent/future.hpp
   include/flightvars/concurrent/promise.hpp
   include/flightvars/io/buffer.hpp
   include/flightvars/io/connection.hpp
   include/flightvars/io/tcp-connection.hpp
   include/flightvars/io/tcp-server.hpp
   include/flightvars/io/types.hpp
   include/flightvars/mqtt/connect.hpp
   include/flightvars/mqtt/codecs/connect.hpp
   include/flightvars/mqtt/codecs/fixed-header.hpp
   include/flightvars/mqtt/codecs/types.hpp
   include/flightvars/mqtt/codecs.hpp
   include/flightvars/mqtt/messages.hpp
   include/flightvars/mqtt/qos.hpp
   include/flightvars/mqtt/session.hpp
   include/flightvars/util/attempt.hpp
   include/flightvars/util/endian.hpp
   include/flightvars/util/exception.hpp
   include/flightvars/util/format.hpp
   include/flightvars/util/logging.hpp
   include/flightvars/util/option.hpp
)

set(flightvars_INCLUDE_DIR
   ${CMAKE_CURRENT_SOURCE_DIR}/include
   PARENT_SCOPE
)

set(flightvars_unittests_SOURCES
   test/concurrent/future-test.cpp
   test/concurrent/promise-test.cpp
   test/io/buffer-test.cpp
   test/io/tcp-server-it.cpp
   test/main.cpp
   test/mqtt/decoder/connect-test.cpp
   test/mqtt/decoder/fixed-header-test.cpp
   test/mqtt/decoder/types-test.cpp
   test/mqtt/encoder/connect-test.cpp
   test/mqtt/encoder/fixed-header-test.cpp
   test/mqtt/encoder/types-test.cpp
   test/mqtt/session-test.cpp
   test/tcp-server-it.cpp
   test/util/attempt-test.cpp
   test/util/format-test.cpp
   test/util/option-test.cpp
)

include_directories(
   ${CMAKE_CURRENT_SOURCE_DIR}/include
   ${Boost_INCLUDE_DIR}
)

add_executable(flightvars_unittests ${flightvars_unittests_SOURCES} ${flightvars_HEADERS})

target_link_libraries(flightvars_unittests ${Boost_LIBRARIES})

enable_testing()
add_test(FlightVarsCommon-UnitTests flightvars_unittests)

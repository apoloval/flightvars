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
   test/concurrent/future-test.hpp
   test/concurrent/promise-test.hpp
   test/io/buffer-test.hpp
   test/io/tcp-server-it.hpp
   test/main.cpp
   test/mqtt/decoder/connect-test.hpp
   test/mqtt/decoder/fixed-header-test.hpp
   test/mqtt/decoder/types-test.hpp
   test/mqtt/encoder/connect-test.hpp
   test/mqtt/encoder/fixed-header-test.hpp
   test/mqtt/encoder/types-test.hpp
   test/mqtt/session-test.hpp
   test/tcp-server-it.hpp
   test/util/attempt-test.hpp
   test/util/format-test.hpp
   test/util/option-test.hpp
)

include_directories(
   ${CMAKE_CURRENT_SOURCE_DIR}/include
   ${Boost_INCLUDE_DIR}
)

add_executable(flightvars_unittests ${flightvars_unittests_SOURCES} ${flightvars_HEADERS})

target_link_libraries(flightvars_unittests ${Boost_LIBRARIES})

enable_testing()
add_test(FlightVarsCommon-UnitTests flightvars_unittests)

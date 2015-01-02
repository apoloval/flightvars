/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <boost/test/unit_test.hpp>

#include <flightvars/io/buffer.hpp>
#include <flightvars/mqtt/codecs/connect.hpp>

using namespace flightvars;
using namespace flightvars::mqtt::codecs;

BOOST_AUTO_TEST_SUITE(MqttDecoderConnect)

BOOST_AUTO_TEST_CASE(MustFailToDecodeOnInvalidProtocolName) {
    auto buff = io::buffer({
        0x00, 0x04, 'A', 'B', 'C', 'D', // protocol name
        0x03,                           // protocol version
        0x00,                           // connect flags
        0x00, 0x0A,                     // keep alive timer
        0x00, 0x03, 'a', 'p', 'v',      // client identifier
    });
    BOOST_CHECK_THROW(decoder<mqtt::connect_message>::decode(buff), decode_error);
}

BOOST_AUTO_TEST_CASE(MustDecode) {
    auto buff = io::buffer({
        0x00, 0x06, 'M', 'Q', 'I', 's', 'd', 'p',   // protocol name
        0x03,                                       // protocol version
        0x00,                                       // connect flags
        0x00, 0x0A,                                 // keep alive timer
        0x00, 0x03, 'a', 'p', 'v',                  // client identifier
    });
    auto msg = decoder<mqtt::connect_message>::decode(buff);

    BOOST_CHECK_EQUAL("apv", msg.get_client_id());
    BOOST_CHECK_EQUAL(10, msg.keep_alive());
    BOOST_CHECK(!msg.get_credentials().is_defined());
    BOOST_CHECK(!msg.get_will().is_defined());
    BOOST_CHECK(!msg.clean_session());

    BOOST_CHECK_EQUAL(0, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustDecodeWithWill) {
    auto buff = io::buffer({
        0x00, 0x06, 'M', 'Q', 'I', 's', 'd', 'p',   // protocol name
        0x03,                                       // protocol version
        0x04,                                       // connect flags
        0x00, 0x0A,                                 // keep alive timer
        0x00, 0x03, 'a', 'p', 'v',                  // client identifier
        0x00, 0x03, 'X', 'Y', 'Z',                  // will topic
        0x00, 0x03, '1', '2', '3',                  // will message
    });
    auto msg = decoder<mqtt::connect_message>::decode(buff);

    BOOST_CHECK_EQUAL("apv", msg.get_client_id());
    BOOST_CHECK_EQUAL(10, msg.keep_alive());
    BOOST_CHECK(!msg.get_credentials().is_defined());
    BOOST_CHECK_EQUAL("XYZ", msg.get_will().get().get_topic());
    BOOST_CHECK_EQUAL("123", msg.get_will().get().get_message());
    BOOST_CHECK(!msg.clean_session());

    BOOST_CHECK_EQUAL(0, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustDecodeWithUsername) {
    auto buff = io::buffer({
        0x00, 0x06, 'M', 'Q', 'I', 's', 'd', 'p',   // protocol name
        0x03,                                       // protocol version
        0x80,                                       // connect flags
        0x00, 0x0A,                                 // keep alive timer
        0x00, 0x03, 'a', 'p', 'v',                  // client identifier
        0x00, 0x03, 'X', 'Y', 'Z',                  // username
    });
    auto msg = decoder<mqtt::connect_message>::decode(buff);

    BOOST_CHECK_EQUAL("apv", msg.get_client_id());
    BOOST_CHECK_EQUAL(10, msg.keep_alive());
    BOOST_CHECK_EQUAL("XYZ", msg.get_credentials().get().get_username());
    BOOST_CHECK(!msg.get_credentials().get().get_password().is_defined());
    BOOST_CHECK(!msg.get_will().is_defined());
    BOOST_CHECK(!msg.clean_session());

    BOOST_CHECK_EQUAL(0, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustDecodeWithUsernameAndPassword) {
    auto buff = io::buffer({
        0x00, 0x06, 'M', 'Q', 'I', 's', 'd', 'p',   // protocol name
        0x03,                                       // protocol version
        0xC0,                                       // connect flags
        0x00, 0x0A,                                 // keep alive timer
        0x00, 0x03, 'a', 'p', 'v',                  // client identifier
        0x00, 0x03, 'X', 'Y', 'Z',                  // username
        0x00, 0x03, '1', '2', '3',                  // password
    });
    auto msg = decoder<mqtt::connect_message>::decode(buff);

    BOOST_CHECK_EQUAL("apv", msg.get_client_id());
    BOOST_CHECK_EQUAL(10, msg.keep_alive());
    BOOST_CHECK_EQUAL("XYZ", msg.get_credentials().get().get_username());
    BOOST_CHECK_EQUAL("123", msg.get_credentials().get().get_password().get());
    BOOST_CHECK(!msg.get_will().is_defined());
    BOOST_CHECK(!msg.clean_session());

    BOOST_CHECK_EQUAL(0, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustDecodeIgnoringUsernameFlagIfFieldIsMissing) {
    auto buff = io::buffer({
        0x00, 0x06, 'M', 'Q', 'I', 's', 'd', 'p',   // protocol name
        0x03,                                       // protocol version
        0x80,                                       // connect flags
        0x00, 0x0A,                                 // keep alive timer
        0x00, 0x03, 'a', 'p', 'v',                  // client identifier
    });
    auto msg = decoder<mqtt::connect_message>::decode(buff);

    BOOST_CHECK_EQUAL("apv", msg.get_client_id());
    BOOST_CHECK_EQUAL(10, msg.keep_alive());
    BOOST_CHECK(!msg.get_credentials().is_defined());
    BOOST_CHECK(!msg.get_will().is_defined());
    BOOST_CHECK(!msg.clean_session());

    BOOST_CHECK_EQUAL(0, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustDecodeIgnoringPasswordFlagIfFieldIsMissing) {
    auto buff = io::buffer({
        0x00, 0x06, 'M', 'Q', 'I', 's', 'd', 'p',   // protocol name
        0x03,                                       // protocol version
        0xC0,                                       // connect flags
        0x00, 0x0A,                                 // keep alive timer
        0x00, 0x03, 'a', 'p', 'v',                  // client identifier
        0x00, 0x03, 'X', 'Y', 'Z',                  // username
    });
    auto msg = decoder<mqtt::connect_message>::decode(buff);

    BOOST_CHECK_EQUAL("apv", msg.get_client_id());
    BOOST_CHECK_EQUAL(10, msg.keep_alive());
    BOOST_CHECK_EQUAL("XYZ", msg.get_credentials().get().get_username());
    BOOST_CHECK(!msg.get_credentials().get().get_password().is_defined());
    BOOST_CHECK(!msg.get_will().is_defined());
    BOOST_CHECK(!msg.clean_session());

    BOOST_CHECK_EQUAL(0, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustFailToDecodeOnPasswordFlagWithMissingUsername) {
    auto buff = io::buffer({
        0x00, 0x06, 'M', 'Q', 'I', 's', 'd', 'p',   // protocol name
        0x03,                                       // protocol version
        0x40,                                       // connect flags
        0x00, 0x0A,                                 // keep alive timer
        0x00, 0x03, 'a', 'p', 'v',                  // client identifier
        0x00, 0x03, '1', '2', '3',                  // password
    });
    BOOST_CHECK_THROW(decoder<mqtt::connect_message>::decode(buff), decode_error);
}

BOOST_AUTO_TEST_SUITE_END()


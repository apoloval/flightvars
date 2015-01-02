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
#include <flightvars/mqtt/codecs/fixed-header.hpp>

using namespace flightvars;
using namespace flightvars::mqtt::codecs;

BOOST_AUTO_TEST_SUITE(MqttFixedHeaderEncoder)

BOOST_AUTO_TEST_CASE(MustEncodeFixedHeader) {
    io::buffer buff;
    mqtt::fixed_header fh = {
        mqtt::message_type::CONNECT, false, mqtt::qos_level::QOS_0, false, 32
    };
    encoder<mqtt::fixed_header>::encode(fh, buff);

    buff.flip();
    BOOST_CHECK_EQUAL(0x10, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0x20, buff.safe_read_value<std::uint8_t>());
}

BOOST_AUTO_TEST_CASE(MustEncodeFixedHeaderWithDupFlagSet) {
    io::buffer buff;
    mqtt::fixed_header fh = {
        mqtt::message_type::CONNECT, true, mqtt::qos_level::QOS_0, false, 32
    };
    encoder<mqtt::fixed_header>::encode(fh, buff);

    buff.flip();
    BOOST_CHECK_EQUAL(0x18, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0x20, buff.safe_read_value<std::uint8_t>());
}

BOOST_AUTO_TEST_CASE(MustEncodeFixedHeaderWithRetainFlagSet) {
    io::buffer buff;
    mqtt::fixed_header fh = {
        mqtt::message_type::CONNECT, false, mqtt::qos_level::QOS_0, true, 32
    };
    encoder<mqtt::fixed_header>::encode(fh, buff);

    buff.flip();
    BOOST_CHECK_EQUAL(0x11, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0x20, buff.safe_read_value<std::uint8_t>());
}

BOOST_AUTO_TEST_CASE(MustEncodeFixedHeaderWithTwoBytesLength) {
    io::buffer buff;
    mqtt::fixed_header fh = {
        mqtt::message_type::CONNECT, false, mqtt::qos_level::QOS_0, false, 321
    };
    encoder<mqtt::fixed_header>::encode(fh, buff);

    buff.flip();
    BOOST_CHECK_EQUAL(0x10, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0xc1, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0x02, buff.safe_read_value<std::uint8_t>());
}

BOOST_AUTO_TEST_CASE(MustEncodeFixedHeaderWithThreeBytesLength) {
    io::buffer buff;
    mqtt::fixed_header fh = {
        mqtt::message_type::CONNECT, false, mqtt::qos_level::QOS_0, false, 32100
    };
    encoder<mqtt::fixed_header>::encode(fh, buff);

    buff.flip();
    BOOST_CHECK_EQUAL(0x10, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0xe4, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0xfa, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0x01, buff.safe_read_value<std::uint8_t>());
}

BOOST_AUTO_TEST_CASE(MustEncodeFixedHeaderWithFourBytesLength) {
    io::buffer buff;
    mqtt::fixed_header fh = {
        mqtt::message_type::CONNECT, false, mqtt::qos_level::QOS_0, false, 3200000
    };
    encoder<mqtt::fixed_header>::encode(fh, buff);

    buff.flip();
    BOOST_CHECK_EQUAL(0x10, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0x80, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0xa8, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0xc3, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0x01, buff.safe_read_value<std::uint8_t>());
}

BOOST_AUTO_TEST_CASE(MustFailToEncodeFixedHeaderWithMoreThanFourBytesLength) {
    io::buffer buff;
    mqtt::fixed_header fh = {
        mqtt::message_type::CONNECT, false, mqtt::qos_level::QOS_0, false, 320000000
    };
    BOOST_CHECK_THROW(encoder<mqtt::fixed_header>::encode(fh, buff), encode_error);
}

BOOST_AUTO_TEST_SUITE_END()

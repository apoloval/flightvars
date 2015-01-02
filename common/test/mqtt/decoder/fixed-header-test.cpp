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

BOOST_AUTO_TEST_SUITE(MqttDecoderFixedHeader)

BOOST_AUTO_TEST_CASE(MustDecodeFixedHeader) {
    auto buff = io::buffer({ 0x10, 0x20 });
    auto fh = decoder<mqtt::fixed_header>::decode(buff);

    BOOST_CHECK_EQUAL(mqtt::message_type::CONNECT, fh.msg_type);
    BOOST_CHECK_EQUAL(false, fh.dup_flag);
    BOOST_CHECK_EQUAL(mqtt::qos_level::QOS_0, fh.qos);
    BOOST_CHECK_EQUAL(false, fh.retain);
    BOOST_CHECK_EQUAL(32, fh.len);
}

BOOST_AUTO_TEST_CASE(MustDecodeFixedHeaderWithDupFlag) {
    auto buff = io::buffer({ 0x18, 0x20 });
    auto fh = decoder<mqtt::fixed_header>::decode(buff);

    BOOST_CHECK_EQUAL(true, fh.dup_flag);
}

BOOST_AUTO_TEST_CASE(MustDecodeFixedHeaderWithRetainFlag) {
    auto buff = io::buffer({ 0x11, 0x20 });
    auto fh = decoder<mqtt::fixed_header>::decode(buff);

    BOOST_CHECK_EQUAL(true, fh.retain);
}

BOOST_AUTO_TEST_CASE(MustDecodeFixedHeaderWithTwoBytesLength) {
    auto buff = io::buffer({ 0x11, 0xc1, 0x02 });
    auto fh = decoder<mqtt::fixed_header>::decode(buff);

    BOOST_CHECK_EQUAL(321, fh.len);
}

BOOST_AUTO_TEST_CASE(MustDecodeFixedHeaderWithThreeBytesLength) {
    auto buff = io::buffer({ 0x11, 0xe4, 0xfa, 0x01 });
    auto fh = decoder<mqtt::fixed_header>::decode(buff);

    BOOST_CHECK_EQUAL(32100, fh.len);
}

BOOST_AUTO_TEST_CASE(MustDecodeFixedHeaderWithFourBytesLength) {
    auto buff = io::buffer({ 0x11, 0x80, 0xa8, 0xc3, 0x01 });
    auto fh = decoder<mqtt::fixed_header>::decode(buff);

    BOOST_CHECK_EQUAL(3200000, fh.len);
}

BOOST_AUTO_TEST_CASE(MustFailToDecodeFixedHeaderWithMoreThanFourBytesLength) {
    auto buff = io::buffer({ 0x11, 0x80, 0x80, 0x80, 0x80, 0x80 });
    BOOST_CHECK_THROW(decoder<mqtt::fixed_header>::decode(buff), decode_error);
    BOOST_CHECK_EQUAL(5, buff.pos());
}

BOOST_AUTO_TEST_SUITE_END()

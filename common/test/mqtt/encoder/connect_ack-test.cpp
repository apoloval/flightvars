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
#include <flightvars/mqtt/codecs/connect_ack.hpp>

using namespace flightvars;
using namespace flightvars::mqtt::codecs;

BOOST_AUTO_TEST_SUITE(MqttEncoderConnectAck)

BOOST_AUTO_TEST_CASE(MustReportEncodeLength) {
    mqtt::connect_ack_message msg {
        mqtt::connect_return_code::SERVER_UNAVAILABLE
    };
    BOOST_CHECK_EQUAL(2, encoder<mqtt::connect_ack_message>::encode_len(msg));
}

BOOST_AUTO_TEST_CASE(MustEncodeSimpleConnectAck) {
    io::buffer buff;
    mqtt::connect_ack_message msg {
        mqtt::connect_return_code::SERVER_UNAVAILABLE
    };
    encoder<mqtt::connect_ack_message>::encode(msg, buff);
    buff.flip();

    BOOST_CHECK_EQUAL(0, buff.safe_read_value<std::uint8_t>()); // reserved byte
    BOOST_CHECK_EQUAL(3, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0, buff.remaining());
}

BOOST_AUTO_TEST_SUITE_END()

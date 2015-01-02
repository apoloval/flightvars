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

BOOST_AUTO_TEST_SUITE(MqttDecoderConnectAck)

BOOST_AUTO_TEST_CASE(MustDecode) {
    auto buff = io::buffer {
        0x00, // reserved
        0x03, // return code
    };
    auto msg = decoder<mqtt::connect_ack_message>::decode(buff);

    BOOST_CHECK_EQUAL(mqtt::connect_return_code::SERVER_UNAVAILABLE, msg.return_code());
    BOOST_CHECK_EQUAL(0, buff.remaining());
}

BOOST_AUTO_TEST_SUITE_END()

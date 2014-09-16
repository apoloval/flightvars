/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <boost/test/unit_test.hpp>

#include <flightvars/io/buffer.hpp>
#include <flightvars/mqtt/codecs/types.hpp>

using namespace flightvars::mqtt::codecs;

BOOST_AUTO_TEST_SUITE(MqttStringEncoder)

BOOST_AUTO_TEST_CASE(MustEncodeString) {
    buffer buff;
    std::string abc = "ABC";
    encoder<std::string>::encode(abc, buff);
    buff.flip();

    BOOST_CHECK_EQUAL(5, buff.remaining());
    BOOST_CHECK_EQUAL(3, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL('A', buff.safe_read_value<char>());
    BOOST_CHECK_EQUAL('B', buff.safe_read_value<char>());
    BOOST_CHECK_EQUAL('C', buff.safe_read_value<char>());
}

BOOST_AUTO_TEST_CASE(MustEncodeLargeString) {
    buffer buff;
    std::string abc = "";
    for (int i = 0; i < 1000; i++) {
        abc += char((i % 26) + 0x41);
    }
    encoder<std::string>::encode(abc, buff);
    buff.flip();

    BOOST_CHECK_EQUAL(1002, buff.remaining());
    BOOST_CHECK_EQUAL(1000, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL('A', buff.safe_read_value<char>());
    BOOST_CHECK_EQUAL('B', buff.safe_read_value<char>());
    BOOST_CHECK_EQUAL('C', buff.safe_read_value<char>());
}

BOOST_AUTO_TEST_SUITE_END()

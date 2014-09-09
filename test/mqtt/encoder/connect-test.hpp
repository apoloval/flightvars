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
#include <flightvars/mqtt/encoder/connect.hpp>

using namespace flightvars::mqtt::encoder;

BOOST_AUTO_TEST_SUITE(MqttEncoderConnect)

BOOST_AUTO_TEST_CASE(MustEncodeSimpleConnect) {
    buffer buff;
    connect_message conn(
        "client",       // client ID
        10,             // keep alive
        false           // clean session
    );
    encoder<connect_message>::encode(conn, buff);
    buff.flip();

    BOOST_CHECK_EQUAL(6, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("MQIsdp", buff.safe_read_string(6));
    BOOST_CHECK_EQUAL(3, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0x00, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(10, from_big_endian(buff.safe_read_value<std::uint16_t>()));

    BOOST_CHECK_EQUAL(6, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("client", buff.safe_read_string(6));

    BOOST_CHECK_EQUAL(0, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustEncodeConnectWithCleanSession) {
    buffer buff;
    connect_message conn(
        "client",       // client ID
        10,             // keep alive
        true            // clean session
    );
    encoder<connect_message>::encode(conn, buff);
    buff.flip();

    BOOST_CHECK_EQUAL(6, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("MQIsdp", buff.safe_read_string(6));
    BOOST_CHECK_EQUAL(3, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0x02, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(10, from_big_endian(buff.safe_read_value<std::uint16_t>()));

    BOOST_CHECK_EQUAL(6, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("client", buff.safe_read_string(6));

    BOOST_CHECK_EQUAL(0, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustEncodeConnectWithUsernameAndPassword) {
    buffer buff;
    connect_message conn(
        "client",       // client ID
        connect_credentials("username", "password"),
        10,             // keep alive
        false           // clean session
    );
    encoder<connect_message>::encode(conn, buff);
    buff.flip();

    BOOST_CHECK_EQUAL(6, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("MQIsdp", buff.safe_read_string(6));
    BOOST_CHECK_EQUAL(3, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0xc0, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(10, from_big_endian(buff.safe_read_value<std::uint16_t>()));

    BOOST_CHECK_EQUAL(6, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("client", buff.safe_read_string(6));
    BOOST_CHECK_EQUAL(8, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("username", buff.safe_read_string(8));
    BOOST_CHECK_EQUAL(8, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("password", buff.safe_read_string(8));

    BOOST_CHECK_EQUAL(0, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustEncodeConnectWithUsernameAndNoPassword) {
    buffer buff;
    connect_message conn(
        "client",       // client ID
        connect_credentials("username"),
        10,             // keep alive
        false           // clean session
    );
    encoder<connect_message>::encode(conn, buff);
    buff.flip();

    BOOST_CHECK_EQUAL(6, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("MQIsdp", buff.safe_read_string(6));
    BOOST_CHECK_EQUAL(3, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0x80, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(10, from_big_endian(buff.safe_read_value<std::uint16_t>()));

    BOOST_CHECK_EQUAL(6, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("client", buff.safe_read_string(6));
    BOOST_CHECK_EQUAL(8, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("username", buff.safe_read_string(8));

    BOOST_CHECK_EQUAL(0, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustEncodeConnectWithWill) {
    buffer buff;
    connect_message conn(
        "client",       // client ID
        connect_will("topic", "message", qos_level::QOS_1, true),
        10,             // keep alive
        false           // clean session
    );
    encoder<connect_message>::encode(conn, buff);
    buff.flip();

    BOOST_CHECK_EQUAL(6, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("MQIsdp", buff.safe_read_string(6));
    BOOST_CHECK_EQUAL(3, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0x2c, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(10, from_big_endian(buff.safe_read_value<std::uint16_t>()));

    BOOST_CHECK_EQUAL(6, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("client", buff.safe_read_string(6));
    BOOST_CHECK_EQUAL(5, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("topic", buff.safe_read_string(5));
    BOOST_CHECK_EQUAL(7, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("message", buff.safe_read_string(7));

    BOOST_CHECK_EQUAL(0, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustEncodeConnectWithAll) {
    buffer buff;
    connect_message conn(
        "client",       // client ID
        connect_credentials("username", "password"),
        connect_will("topic", "message", qos_level::QOS_2, false),
        10,             // keep alive
        false           // clean session
    );
    encoder<connect_message>::encode(conn, buff);
    buff.flip();

    BOOST_CHECK_EQUAL(6, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("MQIsdp", buff.safe_read_string(6));
    BOOST_CHECK_EQUAL(3, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(0xd4, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(10, from_big_endian(buff.safe_read_value<std::uint16_t>()));

    BOOST_CHECK_EQUAL(6, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("client", buff.safe_read_string(6));
    BOOST_CHECK_EQUAL(5, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("topic", buff.safe_read_string(5));
    BOOST_CHECK_EQUAL(7, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("message", buff.safe_read_string(7));
    BOOST_CHECK_EQUAL(8, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("username", buff.safe_read_string(8));
    BOOST_CHECK_EQUAL(8, from_big_endian(buff.safe_read_value<std::uint16_t>()));
    BOOST_CHECK_EQUAL("password", buff.safe_read_string(8));

    BOOST_CHECK_EQUAL(0, buff.remaining());
}

BOOST_AUTO_TEST_SUITE_END()

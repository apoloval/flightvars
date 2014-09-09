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
#include <flightvars/mqtt/session.hpp>

using namespace flightvars::mqtt;

BOOST_AUTO_TEST_SUITE(MqttSession)

struct mock_connection {
    using shared_ptr = std::shared_ptr<mock_connection>;
    buffer read_buffer;
    buffer write_buffer;

    future<shared_buffer> read(const shared_buffer& buff, std::size_t bytes) {
        BOOST_CHECK_EQUAL(bytes, buff->write(read_buffer, bytes));
        read_buffer.inc_pos(bytes);
        return make_future_success(buff);
    }

    template <class Message>
    void prepare_read(const fixed_header& hd, const Message& msg) {
        read_buffer.reset();
        encoder<fixed_header>::encode(hd, read_buffer);
        encoder<Message>::encode(msg, read_buffer);
        read_buffer.flip();
    }
};

std::ostream& operator << (std::ostream& s, const mock_connection& conn) {
    s << "mock connection";
    return s;
}

mock_connection::shared_ptr make_mock_connection() {
    return std::make_shared<mock_connection>();
}

template <class MessageHandler>
mqtt_session<mock_connection>::shared_ptr 
make_session(const mock_connection::shared_ptr& conn,
             const MessageHandler& handler) {
    return make_mqtt_session<mock_connection>(conn, handler);
}

BOOST_AUTO_TEST_CASE(Must)
{
    auto conn = make_mock_connection();
    fixed_header fh = { 
        message_type::CONNECT, false, qos_level::QOS_0, false, 321 };
    connect_message msg("cli0", 30, false);
    conn->prepare_read(fh, msg);

    auto session = make_session(conn, [](const shared_message& msg) {
        
    });
}

BOOST_AUTO_TEST_SUITE_END()

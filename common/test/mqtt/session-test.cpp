/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <boost/test/unit_test.hpp>

#include <flightvars/concurrent/executor.hpp>
#include <flightvars/io/buffer.hpp>
#include <flightvars/mqtt/session.hpp>

using namespace flightvars;
using namespace flightvars::mqtt;

BOOST_AUTO_TEST_SUITE(MqttSession)

struct mock_connection {
    using shared_ptr = std::shared_ptr<mock_connection>;
    io::buffer read_buffer;
    io::buffer write_buffer;

    concurrent::future<io::shared_buffer> read(const io::shared_buffer& buff, std::size_t bytes) {
        BOOST_CHECK_EQUAL(bytes, buff->write(read_buffer, bytes));
        read_buffer.inc_pos(bytes);
        return concurrent::make_future_success(buff);
    }

    template <class Message>
    void prepare_read(const fixed_header& hd, const Message& msg) {
        read_buffer.reset();
        codecs::encoder<fixed_header>::encode(hd, read_buffer);
        codecs::encoder<Message>::encode(msg, read_buffer);
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

template <class MessageHandler, class Executor>
typename mqtt_session<mock_connection, Executor>::shared_ptr
make_session(const mock_connection::shared_ptr& conn,
             const MessageHandler& handler,
             const Executor& exec) {
    return make_mqtt_session<mock_connection>(conn, handler, exec);
}

BOOST_AUTO_TEST_CASE(Must)
{    
    auto conn = make_mock_connection();
    concurrent::asio_service_executor exec;
    fixed_header fh = { 
        message_type::CONNECT, false, qos_level::QOS_0, false, 321 };
    connect_message msg("cli0", 30, false);
    conn->prepare_read(fh, msg);

    auto session = make_session(conn, [](const shared_message& msg) {
        // TODO: return an actual response
        return concurrent::make_future_success(msg);
    }, exec);
    session->start();
    exec.run();
}

BOOST_AUTO_TEST_SUITE_END()

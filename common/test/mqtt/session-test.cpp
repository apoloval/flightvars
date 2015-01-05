/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
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

template <class MessageHandler, class Executor>
typename mqtt_session<mock_connection, Executor>::shared_ptr
make_session(const mock_connection::shared_ptr& conn,
             const MessageHandler& handler,
             const Executor& exec) {
    return make_mqtt_session<mock_connection>(conn, handler, exec);
}

message make_sample_request() {
    return make_connect(
        "cli0", // client ID
        util::make_some<connect_credentials>({ "john.williams", "leia" }),
        util::make_some<connect_will>({ "mytopic", "mymessage", qos_level::QOS_0, false }),
        30,     // keep alive
        false   // clean session
    );
}

BOOST_AUTO_TEST_CASE(MustRoundTripRequestAndResponse) {
    auto conn = make_mock_connection();
    concurrent::asio_service_executor exec;
    auto req_msg = make_sample_request();
    conn->prepare_read_message(req_msg);
    util::option<message> handled_request;

    auto session = make_session(conn, [&](message&& req_msg) {
        handled_request = util::make_some(req_msg);
        return concurrent::make_future_success<message>(
            make_connect_ack(connect_return_code::SERVER_UNAVAILABLE));
    }, exec);
    session->start();
    exec.run();

    auto response = conn->written_message();

    BOOST_CHECK_EQUAL(message_type::CONNECT, handled_request.get().header().msg_type);
    BOOST_CHECK_EQUAL(message_type::CONNACK, response.header().msg_type);
}

BOOST_AUTO_TEST_CASE(MustProcessManyRequestsAndResponsesInSameSession) {
    auto conn = make_mock_connection();
    concurrent::asio_service_executor exec;
    util::option<message> handled_request;
    auto session = make_session(conn, [&](message&& req_msg) {
        handled_request = util::make_some(req_msg);
        return concurrent::make_future_success<message>(
            make_connect_ack(connect_return_code::SERVER_UNAVAILABLE));
    }, exec);
    auto req_msg = make_sample_request();
    conn->prepare_read_messages({ req_msg, req_msg, req_msg });
    session->start();
    exec.run();

    auto response = conn->written_message();

    BOOST_CHECK_EQUAL(message_type::CONNECT, handled_request.get().header().msg_type);
    BOOST_CHECK_EQUAL(message_type::CONNACK, response.header().msg_type);
}

BOOST_AUTO_TEST_SUITE_END()

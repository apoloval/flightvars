/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <boost/test/unit_test.hpp>

#include <flightvars/io/connection.hpp>
#include <flightvars/io/tcp-server.hpp>

using namespace flightvars::io;

BOOST_AUTO_TEST_SUITE(IoTcpServer)

struct server_session : std::enable_shared_from_this<server_session> {

    using shared_ptr = std::shared_ptr<server_session>;

    tcp_connection conn;

    server_session(const tcp_connection& c) : conn(c) {}

    future<void> process() {
        auto input_buffer = make_shared_buffer(3);
        auto self = shared_from_this();
        return read_remaining(conn, input_buffer)
            .fmap<shared_const_buffer>([self](const shared_buffer& buff) {
                buff->flip();
                BOOST_CHECK_EQUAL("APV", buff->data_as_cstr());

                auto output = make_shared_buffer(64);
                output->write("Hello ");
                output->write(*buff);
                output->write("\n");
                output->flip();
                return write_remaining(self->conn, output);
            })
            .map<void>([self](const shared_const_buffer&) {
                // Let the connection die (and close)
            });
    }
};

struct client_session : std::enable_shared_from_this<client_session> {

    using shared_ptr = std::shared_ptr<client_session>;

    tcp_connection conn;

    client_session(const tcp_connection& c) : conn(c) {}

    future<void> process() {
        auto msg = make_shared_buffer("APV");
        msg->flip();
        auto self = shared_from_this();
        return write_remaining(conn, msg)
            .fmap<shared_buffer>([self](const shared_const_buffer& buff) {
                buff->set_pos(0);
                auto reply = make_shared_buffer(10);
                return read_remaining(self->conn, reply); 
            })
            .map<void>([self](const shared_buffer& buff) {
                buff->flip();
                BOOST_CHECK_EQUAL("Hello APV\n", buff->data_as_cstr());
            });
    }
};

BOOST_AUTO_TEST_CASE(MustCommunicateClientAndServer)
{
    executor exec;
    tcp_server server(exec, 5005);
    server.accept()
        .map<server_session::shared_ptr>([](const tcp_connection& conn) {
            return std::make_shared<server_session>(conn);
        })
        .fmap<void>([](const server_session::shared_ptr& session) {
            return session->process();
        });
    auto result = tcp_connect(exec, "localhost", 5005)
        .map<client_session::shared_ptr>([](const tcp_connection& conn) {
            return std::make_shared<client_session>(conn);
        })
        .fmap<void>([](const client_session::shared_ptr& session) {
            return session->process();
        })
        .map<void>([&exec]() {
            exec.stop();
        });
    exec.run();
    BOOST_CHECK_NO_THROW(result.wait_result(std::chrono::milliseconds(50)));
}

BOOST_AUTO_TEST_CASE(MustFailToConnectWhenServerIsNotListening)
{
    executor exec;
    auto result = tcp_connect(exec, "localhost", 5005)
        .map<client_session::shared_ptr>([](const tcp_connection& conn) {
            return std::make_shared<client_session>(conn);
        })
        .fmap<void>([](const client_session::shared_ptr& session) {
            return session->process();
        })
        .map<void>([&exec]() {
            exec.stop();
        });
    exec.run();
    BOOST_CHECK_THROW(result.wait_result(std::chrono::milliseconds(50)), connect_error);
}

BOOST_AUTO_TEST_CASE(MustFailToConnectWhenServerHostIsUnknown)
{
    executor exec;
    auto result = tcp_connect(exec, "abcdefghijklmnopqrstuvwxyz", 5005)
        .map<client_session::shared_ptr>([](const tcp_connection& conn) {
            return std::make_shared<client_session>(conn);
        })
        .fmap<void>([](const client_session::shared_ptr& session) {
            return session->process();
        })
        .map<void>([&exec]() {
            exec.stop();
        });
    exec.run();
    BOOST_CHECK_THROW(result.wait_result(std::chrono::milliseconds(50)), resolve_error);
}

BOOST_AUTO_TEST_SUITE_END()

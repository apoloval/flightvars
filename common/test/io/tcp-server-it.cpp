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
#include <flightvars/io/connection.hpp>
#include <flightvars/io/tcp-server.hpp>

using namespace flightvars;
using namespace flightvars::io;

BOOST_AUTO_TEST_SUITE(IoTcpServer)

struct server_session : std::enable_shared_from_this<server_session> {

    using shared_ptr = std::shared_ptr<server_session>;

    tcp_connection conn;
    buffer input_buffer;
    buffer output_buffer;

    server_session(const tcp_connection& c) : conn(c) {}

    future<void> process() {
        auto self = shared_from_this();
        return conn.read(input_buffer, 3)
            .next<std::size_t>([self](std::size_t) {
                self->input_buffer.flip();
                BOOST_CHECK_EQUAL("APV", self->input_buffer.safe_read_string(3));

                self->input_buffer.flip();
                self->output_buffer.write("Hello ");
                self->output_buffer.write(self->input_buffer);
                self->output_buffer.write("\n");
                self->output_buffer.flip();
                return write_remaining(self->conn, self->output_buffer);
            })
            .then([self](std::size_t) {
                // Let the connection die (and close)
            });
    }
};

struct client_session : std::enable_shared_from_this<client_session> {

    using shared_ptr = std::shared_ptr<client_session>;

    tcp_connection conn;
    buffer input_buffer;
    buffer output_buffer;

    client_session(const tcp_connection& c) : conn(c) {}

    future<void> process() {
        output_buffer.write("APV");
        output_buffer.flip();
        auto self = shared_from_this();
        return write_remaining(conn, output_buffer)
            .next<std::size_t>([self](std::size_t) {
                self->output_buffer.set_pos(0);
                return self->conn.read(self->input_buffer, 10);
            })
            .then([self](std::size_t) {
                self->input_buffer.flip();
                BOOST_CHECK_EQUAL("Hello APV\n", self->input_buffer.safe_read_string(10));
            });
    }
};

BOOST_AUTO_TEST_CASE(MustCommunicateClientAndServer)
{
    concurrent::asio_service_executor exec;
    tcp_server server(5005, exec);
    auto srv = server.accept()
        .then([](const tcp_connection& conn) {
            return std::make_shared<server_session>(conn);
        })
        .next<void>([](const server_session::shared_ptr& session) {
            return session->process();
        });
    auto result = tcp_connect("localhost", 5005, exec)
        .then([](const tcp_connection& conn) {
            return std::make_shared<client_session>(conn);
        })
        .next<void>([](const client_session::shared_ptr& session) {
            return session->process();
        })
        .then([exec]() mutable {
            exec.stop();
        });
    exec.run();
    BOOST_CHECK_NO_THROW(result.wait_for(std::chrono::milliseconds(50)));
}

BOOST_AUTO_TEST_CASE(MustFailToConnectWhenServerIsNotListening)
{
    concurrent::asio_service_executor exec;
    auto result = tcp_connect("localhost", 5005, exec)
        .then([](const tcp_connection& conn) {
            return std::make_shared<client_session>(conn);
        })
        .next<void>([](const client_session::shared_ptr& session) {
            return session->process();
        })
        .then([exec]() mutable {
            exec.stop();
        });
    exec.run();
    BOOST_CHECK_THROW(result.get_for(std::chrono::milliseconds(50)), connect_error);
}

BOOST_AUTO_TEST_CASE(MustFailToConnectWhenServerHostIsUnknown)
{
    concurrent::asio_service_executor exec;
    auto result = tcp_connect("abcdefghijklmnopqrstuvwxyz", 5005, exec)
        .then([](const tcp_connection& conn) {
            return std::make_shared<client_session>(conn);
        })
        .next<void>([](const client_session::shared_ptr& session) {
            return session->process();
        })
        .then([exec]() mutable {
            exec.stop();
        });
    exec.run();
    BOOST_CHECK_THROW(result.get_for(std::chrono::milliseconds(50)), resolve_error);
}

BOOST_AUTO_TEST_SUITE_END()

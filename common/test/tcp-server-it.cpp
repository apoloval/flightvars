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
#include <flightvars/io/tcp-server.hpp>

using namespace flightvars;
using namespace flightvars::io;

BOOST_AUTO_TEST_SUITE(IoTcpServer)

BOOST_AUTO_TEST_CASE(Must)
{
    concurrent::asio_service_executor exec;
    /*
    tcp_server server(5005, [](tcp_connection& conn) {
        auto input_buffer = io::make_shared_buffer(256);
        auto output_buffer = "Hello World!\n";

        conn.read(input_buffer, 256);
            .fmap<std::size_t>([conn, input_buffer, output_buffer](
                    const std::size_t&) {
                std::cerr << "Client says: " << input_buffer << std::endl;
                return conn.write(output_buffer, 13);
            });
    }, exec);
    // exec.run();*/
}

BOOST_AUTO_TEST_SUITE_END()

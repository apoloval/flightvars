/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_IO_TCP_SERVER_H
#define FLIGHTVARS_IO_TCP_SERVER_H

#include <functional>

#include <boost/asio.hpp>
#include <boost/format.hpp>

#include <flightvars/concurrent/executor.hpp>
#include <flightvars/io/tcp-connection.hpp>
#include <flightvars/io/types.hpp>
#include <flightvars/util/logging.hpp>

namespace flightvars { namespace io {

class tcp_server {
public:

    FV_DECL_EXCEPTION(accept_error);

    tcp_server(int port, concurrent::asio_service_executor& exec) :
        _acceptor(exec.io_service(), endpoint(tcp::v4(), port)) {}

    future<tcp_connection> accept() {
        auto socket = std::make_shared<tcp::socket>(
            _acceptor.get_io_service());        
        auto result = std::make_shared<promise<tcp_connection>>();
        auto handler = std::bind(
            &tcp_server::accepted, 
            this, 
            result,
            socket, 
            std::placeholders::_1);
        _acceptor.async_accept(*socket, handler);
        return result->get_future();
    }

private:

    mutable util::logger _log;
    tcp::acceptor _acceptor;

    void accepted(const std::shared_ptr<promise<tcp_connection>>& promised,
                  const shared_socket& socket, 
                  const boost::system::error_code& error) {
        if (!error) {
            BOOST_LOG_SEV(_log, util::log_level::TRACE) <<
                "Accepted TCP connection from " << socket->remote_endpoint() << 
                " to " << socket->local_endpoint();
            promised->set_value(tcp_connection(socket));
        } else {
            auto msg = boost::format("Unexpected error while accepting TCP connections on %s") % 
                _acceptor.local_endpoint();
            BOOST_LOG_SEV(_log, util::log_level::ERROR) << msg;
            promised->set_failure(accept_error(boost::str(msg)));
        }
    }
};

}}

#endif

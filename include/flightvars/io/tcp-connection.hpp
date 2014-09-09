/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_IO_TCP_CONNECTION_H
#define FLIGHTVARS_IO_TCP_CONNECTION_H

#include <functional>

#include <boost/asio.hpp>
#include <boost/format.hpp>

#include <flightvars/io/buffer.hpp>
#include <flightvars/io/types.hpp>
#include <flightvars/concurrent/promise.hpp>
#include <flightvars/concurrent/future.hpp>
#include <flightvars/util/logging.hpp>

namespace flightvars { namespace io {

using namespace flightvars::concurrent;

class tcp_connection {
public:
    
    FV_DECL_EXCEPTION(read_error);
    FV_DECL_EXCEPTION(write_error);

    using shared_ptr = std::shared_ptr<tcp_connection>;

    tcp_connection(const shared_socket& socket) : _socket(socket) {}

    std::string str() const {
        return boost::str(
            boost::format("TCP connection (%s -> %s)") % 
                _socket->local_endpoint() % _socket->remote_endpoint());
    }

    future<shared_buffer> read(const shared_buffer& buff, std::size_t bytes) {
        auto promise = make_shared_promise<shared_buffer>();
        auto handler = std::bind(
            &tcp_connection::handle_read, 
            this, 
            buff,
            promise, 
            std::placeholders::_1, 
            std::placeholders::_2);
        boost::asio::async_read(
            *_socket, boost::asio::buffer(buff->to_boost_asio(bytes)), handler);
        return make_future(*promise);
    }

    future<shared_const_buffer> write(const shared_const_buffer& buff, std::size_t bytes) {
        auto promise = make_shared_promise<shared_const_buffer>();
        auto handler = std::bind(
            &tcp_connection::handle_write, 
            this, 
            buff,
            promise, 
            std::placeholders::_1, 
            std::placeholders::_2);
        boost::asio::async_write(
            *_socket, boost::asio::buffer(buff->to_boost_asio(bytes)), handler);
        return make_future(*promise);
    } 

private:

    mutable util::logger _log;
    shared_socket _socket;

    void handle_read(const shared_buffer& buff,
                     const std::shared_ptr<promise<shared_buffer>>& promise,
                     const boost::system::error_code& error,
                     std::size_t bytes_transferred) {
        buff->inc_pos(bytes_transferred);
        if (!error) {
            promise->set_success(buff);
        } else {
            auto msg = boost::format("Unexpected error while reading from %s: %s") % 
                str() % error;
            BOOST_LOG_SEV(_log, log_level::WARN) << msg;
            promise->set_failure(read_error(boost::str(msg)));
        }
    }

    void handle_write(const shared_const_buffer& buff,
                      const std::shared_ptr<promise<shared_const_buffer>>& promise,
                      const boost::system::error_code& error,
                      std::size_t bytes_transferred) {
        buff->inc_pos(bytes_transferred);
        if (!error) {
            promise->set_success(buff);
        } else {
            auto msg = boost::format("Unexpected error while writing to %s: %s") % 
                str() % error;
            BOOST_LOG_SEV(_log, log_level::WARN) << msg;
            promise->set_failure(write_error(boost::str(msg)));
        }
    }
};

using shared_tcp_connection = std::shared_ptr<tcp_connection>;

std::ostream& operator << (std::ostream& s, const tcp_connection& conn) {
    s << conn.str();
    return s;
}

FLIGHTVARS_DECL_EXCEPTION(resolve_error);

future<tcp::resolver::iterator> resolve(executor& exec, 
                                        const std::string& host, 
                                        std::uint32_t port) {
    auto resolver = std::make_shared<tcp::resolver>(exec);
    auto result = make_shared_promise<tcp::resolver::iterator>();
    resolver->async_resolve(
        { host, std::to_string(port) },
        [resolver, result, host, port](const boost::system::error_code& error,
                           tcp::resolver::iterator it) {
            logger log;
            if (!error) {
                result->set_success(it);
            } else {
                auto msg = boost::format(
                    "Unexpected error ocurred while resolving %s:%d: %s") %
                    host % port % error.message();
                BOOST_LOG_SEV(log, log_level::ERROR) << msg;
                result->set_failure(resolve_error(msg.str()));
            }
        });
    return make_future(*result);
}

FLIGHTVARS_DECL_EXCEPTION(connect_error);

future<tcp_connection> tcp_connect(executor& exec, 
                                   const std::string& host, 
                                   std::uint32_t port) {
    return resolve(exec, host, port)
        .fmap<tcp_connection>([&exec, host, port](const tcp::resolver::iterator& ep_it) {
            auto socket = std::make_shared<tcp::socket>(exec);
            auto result = make_shared_promise<tcp_connection>();
            boost::asio::async_connect(
                *socket, 
                ep_it, 
                [socket, result, host, port](const boost::system::error_code& error, 
                                 tcp::resolver::iterator) {
                    logger log;
                    if (!error) {
                        auto conn = tcp_connection(socket);
                        BOOST_LOG_SEV(log, log_level::TRACE) << "Established new " << conn.str();
                        result->set_success(conn);
                    } else {
                        auto msg = boost::format(
                            "Unexpected error ocurred while connecting to TCP endpoint %s:%d: %s") %
                            host % port % error.message();
                        BOOST_LOG_SEV(log, log_level::ERROR) << msg;
                        result->set_failure(connect_error(boost::str(msg)));
                    }
                });        
            return make_future(*result);
        });
}

}}

#endif

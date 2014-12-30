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
#include <string>

#include <boost/asio.hpp>
#include <boost/format.hpp>

#include <flightvars/io/buffer.hpp>
#include <flightvars/io/types.hpp>
#include <flightvars/concurrent/executor.hpp>
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
        auto p = std::make_shared<promise<shared_buffer>>();
        auto handler = std::bind(
            &tcp_connection::handle_read, 
            this, 
            buff,
            p,
            std::placeholders::_1, 
            std::placeholders::_2);
        boost::asio::async_read(
            *_socket, boost::asio::buffer(buff->to_boost_asio(bytes)), handler);
        return p->get_future();
    }

    future<shared_const_buffer> write(const shared_const_buffer& buff, std::size_t bytes) {
        auto p = std::make_shared<promise<shared_const_buffer>>();
        auto handler = std::bind(
            &tcp_connection::handle_write, 
            this, 
            buff,
            p,
            std::placeholders::_1, 
            std::placeholders::_2);
        boost::asio::async_write(
            *_socket, boost::asio::buffer(buff->to_boost_asio(bytes)), handler);
        return p->get_future();
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
            promise->set_value(buff);
        } else {
            auto msg = boost::format("Unexpected error while reading from %s: %s") % 
                str() % error;
            BOOST_LOG_SEV(_log, util::log_level::WARN) << msg;
            promise->set_failure(read_error(boost::str(msg)));
        }
    }

    void handle_write(const shared_const_buffer& buff,
                      const std::shared_ptr<promise<shared_const_buffer>>& promise,
                      const boost::system::error_code& error,
                      std::size_t bytes_transferred) {
        buff->inc_pos(bytes_transferred);
        if (!error) {
            promise->set_value(buff);
        } else {
            auto msg = boost::format("Unexpected error while writing to %s: %s") % 
                str() % error;
            BOOST_LOG_SEV(_log, util::log_level::WARN) << msg;
            promise->set_failure(write_error(boost::str(msg)));
        }
    }
};

using shared_tcp_connection = std::shared_ptr<tcp_connection>;

inline std::ostream& operator << (std::ostream& s, const tcp_connection& conn) {
    s << conn.str();
    return s;
}

FLIGHTVARS_DECL_EXCEPTION(resolve_error);

inline future<tcp::resolver::iterator>
resolve(const std::string& host,
        std::uint32_t port,
        const concurrent::asio_service_executor& exec) {
    auto resolver = std::make_shared<tcp::resolver>(exec.io_service());
    auto result = std::make_shared<promise<tcp::resolver::iterator>>();
    resolver->async_resolve(
        { host, std::to_string(port) },
        [resolver, result, host, port](const boost::system::error_code& error,
                                       tcp::resolver::iterator it) {
            util::logger log;
            if (!error) {
                result->set_value(it);
            } else {
                auto msg = boost::format(
                    "Unexpected error ocurred while resolving %s:%d: %s") %
                    host % port % error.message();
                BOOST_LOG_SEV(log, util::log_level::ERROR) << msg;
                result->set_failure(resolve_error(msg.str()));
            }
        });
    return result->get_future();
}

FLIGHTVARS_DECL_EXCEPTION(connect_error);

inline future<tcp_connection>
tcp_connect(const std::string& host,
            std::uint32_t port,
            const concurrent::asio_service_executor& exec) {
    return resolve(host, port, exec)
        .next<tcp_connection>([host, port, exec](const tcp::resolver::iterator& ep_it) {
            auto socket = std::make_shared<tcp::socket>(exec.io_service());
            auto result = std::make_shared<promise<tcp_connection>>();
            boost::asio::async_connect(
                *socket, 
                ep_it, 
                [socket, result, host, port, exec](const boost::system::error_code& error,
                                                   tcp::resolver::iterator) {
                    util::logger log;
                    if (!error) {
                        auto conn = tcp_connection(socket);
                        BOOST_LOG_SEV(
                            log, util::log_level::TRACE) << "Established new " << conn.str();
                        result->set_value(conn);
                    } else {
                        auto msg = boost::format(
                            "Unexpected error ocurred while connecting to TCP endpoint %s:%d: %s") %
                            host % port % error.message();
                        BOOST_LOG_SEV(log, util::log_level::ERROR) << msg;
                        result->set_failure(connect_error(boost::str(msg)));
                    }
                });        
            return result->get_future();
        });
}

}}

#endif

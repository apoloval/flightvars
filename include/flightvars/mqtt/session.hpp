/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_MQTT_SESSION_H
#define FLIGHTVARS_MQTT_SESSION_H

#include <memory>

#include <flightvars/io/connection.hpp>
#include <flightvars/mqtt/decoder.hpp>
#include <flightvars/mqtt/encoder.hpp>
#include <flightvars/mqtt/messages.hpp>
#include <flightvars/util/logging.hpp>

namespace flightvars { namespace mqtt {

FLIGHTVARS_DECL_EXCEPTION(session_error);

template <class Connection>
class mqtt_session : public std::enable_shared_from_this<mqtt_session<Connection>> {
public:

    using shared_ptr = std::shared_ptr<mqtt_session>;

    template <class C, class MessageHandler>
    friend typename mqtt_session<C>::shared_ptr make_mqtt_session(
        const typename C::shared_ptr&, const MessageHandler&);

private:

    util::logger _log;
    typename Connection::shared_ptr _conn;
    std::function<future<shared_message>(const shared_message&)> _msg_handler;

    template <class MessageHandler>
    mqtt_session(const typename Connection::shared_ptr& conn, 
                 const MessageHandler& msg_handler) : _conn(conn), _msg_handler(msg_handler) {}

    void start() {
        BOOST_LOG_SEV(_log, log_level::DEBUG) << "Initializing a new MQTT session on " << *_conn;
        auto buff = make_shared_buffer();
        process_request(buff);
    }

    void process_request(const shared_buffer& buff) {
        BOOST_LOG_SEV(_log, log_level::TRACE) << 
            "Processing new request for session on " << *_conn;
        process_message(buff).add_listener(
            std::bind(&mqtt_session::request_processed, 
                this->shared_from_this(), buff, std::placeholders::_1));
    }

    void request_processed(const shared_buffer& buff, const attempt<void>& result) {
        try { 
            result.get();
            BOOST_LOG_SEV(_log, log_level::DEBUG) << 
                "Request successfully processed on " << *_conn;
        } catch (const std::exception& e) {
            BOOST_LOG_SEV(_log, log_level::ERROR) << 
                "Error while processing request on " << *_conn << ": " << e.what();
        }
    }

    future<void> process_message(const shared_buffer& buff) {
        return receive_message(buff)
            .template fmap<shared_message>(
                std::bind(&mqtt_session::deliver_message, 
                    this->shared_from_this(), buff, std::placeholders::_1))
            .template fmap<shared_message>(
                std::bind(&mqtt_session::send_message, 
                    this->shared_from_this(), buff, std::placeholders::_1))
            .template map<void>(
                std::bind(&mqtt_session::message_processed, 
                    this->shared_from_this(), buff, std::placeholders::_1));
    }

    future<shared_message> receive_message(const shared_buffer& buff) {
        BOOST_LOG_SEV(_log, log_level::TRACE) << "Receiving a new message from connection";
        return read_header(buff)
            .template fmap<shared_message>(
                std::bind(&mqtt_session::read_complete_message, 
                    this->shared_from_this(), buff, std::placeholders::_1));
    }

    future<fixed_header> read_header(const shared_buffer& buff) {
        return _conn->read(buff, fixed_header::BASE_LEN)
            .template fmap<fixed_header>(
                std::bind(&mqtt_session::fixed_header_read, 
                    this->shared_from_this(), std::placeholders::_1, 1));
    }

    future<fixed_header> fixed_header_read(const shared_buffer& buff, std::size_t size_bytes) {
        buff->flip();
        bool bytes_follow = (buff->last() & 0x80) && size_bytes < 4;
        if (bytes_follow) {
            BOOST_LOG_SEV(_log, log_level::TRACE) << 
                "Fixed header is incomplete, some byte(s) follow; reading one more byte... ";
            buff->reset();
            buff->set_pos(size_bytes + 1);
            return _conn->read(buff, 1)
                .template fmap<fixed_header>(
                    std::bind(&mqtt_session::fixed_header_read, 
                        this->shared_from_this(), std::placeholders::_1, size_bytes + 1));
        } else {
            auto header = decoder::decoder<fixed_header>::decode(*buff);
            BOOST_LOG_SEV(_log, log_level::TRACE) << "Fixed header read: " << header;
            return make_future_success(header);
        }
    }

    future<shared_message> read_complete_message(const shared_buffer& buff, 
                                                 const fixed_header& header) {
        buff->reset();
        return _conn->read(buff, header.len)
            .template map<shared_message>(
                std::bind(&mqtt_session::process_content,
                    this->shared_from_this(), header, std::placeholders::_1));
    }

    shared_message process_content(const fixed_header& header, const shared_buffer& buff) {
        buff->flip();
        auto expected_len = header.len;
        auto actual_len = buff->remaining();
        if (actual_len != expected_len) {
            throw session_error(format(
                "cannot process MQTT message content: "
                "expected %d bytes of remaining length, but %d found", expected_len, actual_len));
        }
        switch (header.msg_type) {
            case message_type::CONNECT: {
                auto connect = decoder::decoder<connect_message>::decode(*buff);
                return std::make_shared<message>(header, connect);
                break;
            }
            default:
                throw std::runtime_error(format("cannot decode message of unknown type %s", 
                    message_type_str(header.msg_type)));
        }
    }

    future<shared_message> deliver_message(const shared_buffer& buff, const shared_message& msg) {
        return _msg_handler(msg);
    }

    future<shared_message> send_message(const shared_buffer& buff, const shared_message& msg) {
        throw std::runtime_error("send_message is not implemented");
    }

    void message_processed(const shared_buffer& buff, const shared_message& msg) {
        BOOST_LOG_SEV(_log, log_level::DEBUG) << "Message " << *msg << " successfully processed";
    }

};

template <class Connection, class MessageHandler>
typename mqtt_session<Connection>::shared_ptr 
make_mqtt_session(const typename Connection::shared_ptr& conn,
                  const MessageHandler& msg_handler) {
    auto session = std::shared_ptr<mqtt_session<Connection>>(
        new mqtt_session<Connection>(conn, msg_handler));
    session->start();
    return session;
}

}}

#endif

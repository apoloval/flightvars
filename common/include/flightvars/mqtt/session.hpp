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
#include <flightvars/mqtt/codecs.hpp>
#include <flightvars/mqtt/messages.hpp>
#include <flightvars/util/logging.hpp>

#include "mock_connection.hpp"

namespace flightvars { namespace mqtt {

FLIGHTVARS_DECL_EXCEPTION(session_error);

/**
 * A MQTT session.
 *
 * The MQTT session wraps an IO Connection object and implements the logic to extract MQTT
 * requests, deliver them to a handler and use the resulting response message to write it back
 * to the connection.
 *
 * On its constructor, the Connection object, the handler and one executor are provided.
 * The handler is a function that receives a `mqtt::shared_message` and returns
 * `concurrent::future<mqtt::shared_message>`. That handler is expected to process the incoming
 * message and produce a future response message. The executor passed as argument is used to
 * execute the read and write actions and the handler.
 *
 * This is a create-and-forget class. Its private constructor prevents it to be instantiated as a
 * stack variable. Use the convenience function `make_mqtt_session()` to obtain a shared instance.
 * Once `mqtt_session::start()` is invoked, it will continue running while the given connection
 * is alive, regardless the shared pointer is disposed or not.
 */
template <class Connection, class Executor>
class mqtt_session : public std::enable_shared_from_this<mqtt_session<Connection, Executor>> {
public:

    using shared_ptr = std::shared_ptr<mqtt_session>;

    template <class C, class MessageHandler, class E>
    friend typename mqtt_session<C, E>::shared_ptr make_mqtt_session(
        const typename C::shared_ptr&,
        const MessageHandler&,
        const E&);

    void start() {
        BOOST_LOG_SEV(_log, util::log_level::DEBUG) <<
            "Initializing a new MQTT session on " << *_conn;
        concurrent::run(_exec, &mqtt_session::process_request, self());
    }

private:

    util::logger _log;
    typename Connection::shared_ptr _conn;
    std::function<concurrent::future<shared_message>(const shared_message&)> _msg_handler;
    Executor _exec;
    io::buffer _input_buff;
    io::buffer _output_buff;

    template <class MessageHandler>
    mqtt_session(const typename Connection::shared_ptr& conn, 
                 const MessageHandler& msg_handler,
                 const Executor& exec) : _conn(conn), _msg_handler(msg_handler), _exec(exec) {}

    std::shared_ptr<mqtt_session> self() { return this->shared_from_this(); }

    void process_request() {
        using namespace std::placeholders;

        BOOST_LOG_SEV(_log, util::log_level::TRACE) <<
            "Expecting new request for session on " << *_conn;
        read_request()
            .next<shared_message>(_msg_handler, _exec)
            .next<void>(std::bind(&mqtt_session::write_response, self(), _1), _exec)
            .finally(std::bind(&mqtt_session::request_processed, self(), _1), _exec);
    }

    concurrent::future<shared_message> read_request() {
        _input_buff.reset();
        return read_header()
            .next<shared_message>(std::bind(
                &mqtt_session::read_message_from_header, self(), std::placeholders::_1));
    }

    concurrent::future<void> write_response(const shared_message& response) {
        _output_buff.reset();
        BOOST_LOG_SEV(_log, util::log_level::DEBUG) <<
            "Response message encoded to " << *_conn << ": " << *response;
        encode(*response, _output_buff);
        _output_buff.flip();
        return io::write_remaining(*_conn, _output_buff)
            .then([](std::size_t) {});
    }

    void request_processed(const util::attempt<void>& result) {
        try { 
            result.get();
            BOOST_LOG_SEV(_log, util::log_level::DEBUG) << 
                "Request successfully processed on " << *_conn;
            concurrent::run(_exec, &mqtt_session::process_request, self());
        } catch (const std::exception& e) {
            BOOST_LOG_SEV(_log, util::log_level::ERROR) << 
                "Error while processing request on " << *_conn << ": " << e.what();
        }
    }        

    concurrent::future<fixed_header>
    read_header() {
        using namespace std::placeholders;

        return _conn->read(_input_buff, fixed_header::BASE_LEN)
            .next<fixed_header>(std::bind(&mqtt_session::decode_header, self(), _1, 1));
    }

    concurrent::future<fixed_header>
    decode_header(std::size_t bytes_read,
                  std::size_t size_bytes) {
        using namespace std::placeholders;

        _input_buff.flip();
        bool bytes_follow = (_input_buff.last() & 0x80) && size_bytes < 4;
        if (bytes_follow) {
            BOOST_LOG_SEV(_log, util::log_level::TRACE) << 
                "Fixed header from " << *_conn <<
                " is incomplete, some byte(s) follow; reading one more byte... ";
            _input_buff.reset();
            _input_buff.set_pos(size_bytes + 1);
            return _conn->read(_input_buff, 1).next<fixed_header>(
                std::bind(&mqtt_session::decode_header, self(), _1, size_bytes + 1));
        } else {
            auto header = codecs::decoder<fixed_header>::decode(_input_buff);
            BOOST_LOG_SEV(_log, util::log_level::TRACE) <<
                "Fixed header read from " << *_conn << ": " << header;
            return concurrent::make_future_success(std::move(header));
        }
    }

    concurrent::future<shared_message>
    read_message_from_header(const fixed_header& header) {
        using namespace std::placeholders;

        _input_buff.reset();
        return _conn->read(_input_buff, header.len)
            .then(std::bind(&mqtt_session::decode_content, self(), header, _1));
    }

    shared_message decode_content(const fixed_header& header,
                                  std::size_t bytes_read) {
        _input_buff.flip();
        auto expected_len = header.len;
        auto actual_len = _input_buff.remaining();
        if (actual_len != expected_len) {
            throw session_error(util::format(
                "cannot process MQTT message content: "
                "expected %d bytes of remaining length, but %d found", expected_len, actual_len));
        }
        auto msg = decode(header, _input_buff);
        BOOST_LOG_SEV(_log, util::log_level::DEBUG) <<
            "Request message decoded from " << *_conn << ": " << *msg;
        return msg;
    }
};

template <class Connection, class MessageHandler, class Executor>
typename mqtt_session<Connection, Executor>::shared_ptr
make_mqtt_session(const typename Connection::shared_ptr& conn,
                  const MessageHandler& msg_handler,
                  const Executor& exec) {
    using session_type = mqtt_session<Connection, Executor>;
    auto session = std::shared_ptr<session_type>(new session_type(conn, msg_handler, exec));
    return session;
}

}}

#endif

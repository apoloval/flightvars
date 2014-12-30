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
        auto buff = io::make_shared_buffer();
        concurrent::run(_exec, &mqtt_session::process_request, self(), buff);
    }

private:

    util::logger _log;
    typename Connection::shared_ptr _conn;
    std::function<concurrent::future<shared_message>(const shared_message&)> _msg_handler;
    Executor _exec;

    template <class MessageHandler>
    mqtt_session(const typename Connection::shared_ptr& conn, 
                 const MessageHandler& msg_handler,
                 const Executor& exec) : _conn(conn), _msg_handler(msg_handler), _exec(exec) {}

    std::shared_ptr<mqtt_session> self() { return this->shared_from_this(); }

    void process_request(const io::shared_buffer& buff) {
        using namespace std::placeholders;

        BOOST_LOG_SEV(_log, util::log_level::TRACE) <<
            "Expecting new request for session on " << *_conn;
        read_request(buff)
            .next<shared_message>(_msg_handler, _exec)
            .next<void>(std::bind(&mqtt_session::write_response, self(), buff, _1), _exec)
            .finally(std::bind(&mqtt_session::request_processed, self(), buff, _1), _exec);
    }

    concurrent::future<shared_message> read_request(const io::shared_buffer& buff) {
        return read_header(buff)
            .next<shared_message>(std::bind(
                &mqtt_session::read_message_from_header, self(), buff, std::placeholders::_1));
    }

    concurrent::future<void> write_response(const io::shared_buffer& buff,
                                            const shared_message& response) {
        BOOST_LOG_SEV(_log, util::log_level::DEBUG) <<
            "Replying to " << *_conn << " with message " << *response;
        // TODO: actually write response
        return concurrent::make_future_success<void>();
    }

    void request_processed(const io::shared_buffer& buff, const util::attempt<void>& result) {
        try { 
            result.get();
            BOOST_LOG_SEV(_log, util::log_level::DEBUG) << 
                "Request successfully processed on " << *_conn;
            concurrent::run(_exec, &mqtt_session::process_request, self(), buff);
        } catch (const std::exception& e) {
            BOOST_LOG_SEV(_log, util::log_level::ERROR) << 
                "Error while processing request on " << *_conn << ": " << e.what();
        }
    }        

    concurrent::future<fixed_header>
    read_header(const io::shared_buffer& buff) {
        using namespace std::placeholders;

        return _conn->read(buff, fixed_header::BASE_LEN)
            .next<fixed_header>(std::bind(&mqtt_session::decode_header, self(), _1, 1));
    }

    concurrent::future<fixed_header>
    decode_header(const io::shared_buffer& buff, std::size_t size_bytes) {
        using namespace std::placeholders;

        buff->flip();
        bool bytes_follow = (buff->last() & 0x80) && size_bytes < 4;
        if (bytes_follow) {
            BOOST_LOG_SEV(_log, util::log_level::TRACE) << 
                "Fixed header from " << *_conn <<
                " is incomplete, some byte(s) follow; reading one more byte... ";
            buff->reset();
            buff->set_pos(size_bytes + 1);
            return _conn->read(buff, 1).next<fixed_header>(
                std::bind(&mqtt_session::decode_header, self(), _1, size_bytes + 1));
        } else {
            auto header = codecs::decoder<fixed_header>::decode(*buff);
            BOOST_LOG_SEV(_log, util::log_level::TRACE) <<
                "Fixed header read from " << *_conn << ": " << header;
            return concurrent::make_future_success(std::move(header));
        }
    }

    concurrent::future<shared_message>
    read_message_from_header(const io::shared_buffer& buff, const fixed_header& header) {
        using namespace std::placeholders;

        buff->reset();
        return _conn->read(buff, header.len)
            .then(std::bind(&mqtt_session::decode_content, self(), header, _1));
    }

    shared_message decode_content(const fixed_header& header, const io::shared_buffer& buff) {
        buff->flip();
        auto expected_len = header.len;
        auto actual_len = buff->remaining();
        if (actual_len != expected_len) {
            throw session_error(util::format(
                "cannot process MQTT message content: "
                "expected %d bytes of remaining length, but %d found", expected_len, actual_len));
        }
        auto msg = decode(header, *buff);
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

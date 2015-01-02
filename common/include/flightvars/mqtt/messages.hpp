/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_MQTT_MESSAGES_H
#define FLIGHTVARS_MQTT_MESSAGES_H

#include <cinttypes>

#include <boost/format.hpp>

#include <flightvars/mqtt/connect.hpp>
#include <flightvars/mqtt/connect_ack.hpp>
#include <flightvars/mqtt/qos.hpp>
#include <flightvars/mqtt/codecs/types.hpp>
#include <flightvars/util/exception.hpp>
#include <flightvars/util/option.hpp>

namespace flightvars { namespace mqtt {

enum class message_type {
    RESERVED_0  = 0,
    CONNECT     = 1,
    CONNACK     = 2,
    PUBLISH     = 3,
    PUBACK      = 4,
    PUBREC      = 5,
    PUBREL      = 6,
    PUBCOMP     = 7,
    SUBSCRIBE   = 8,
    SUBACK      = 9,
    UNSUBSCRIBE = 10,
    UNSUBACK    = 11,
    PINGREQ     = 12,
    PINGRESP    = 13,
    DISCONNECT  = 14,
    RESERVED_15 = 15
};

inline std::string message_type_str(message_type mt) {
    switch (mt) {
        case message_type::RESERVED_0:    return "RESERVED_0";
        case message_type::CONNECT:       return "CONNECT";
        case message_type::CONNACK:       return "CONNACK";
        case message_type::PUBLISH:       return "PUBLISH";
        case message_type::PUBACK:        return "PUBACK";
        case message_type::PUBREC:        return "PUBREC";
        case message_type::PUBREL:        return "PUBREL";
        case message_type::PUBCOMP:       return "PUBCOMP";
        case message_type::SUBSCRIBE:     return "SUBSCRIBE";
        case message_type::SUBACK:        return "SUBACK";
        case message_type::UNSUBSCRIBE:   return "UNSUBSCRIBE";
        case message_type::UNSUBACK:      return "UNSUBACK";
        case message_type::PINGREQ:       return "PINGREQ";
        case message_type::PINGRESP:      return "PINGRESP";
        case message_type::DISCONNECT:    return "DISCONNECT";
        case message_type::RESERVED_15:   return "RESERVED_15";
        default:                          return "UNKNOWN";
    }
}

inline std::ostream& operator << (std::ostream& s, const message_type& mt) {
    s << message_type_str(mt);
    return s;
}

struct fixed_header {

    static constexpr std::size_t BASE_LEN = 2;

    message_type msg_type;
    bool dup_flag;
    qos_level qos;
    bool retain;
    std::size_t len;

    std::string str() const {
        return boost::str(boost::format("{ type: %s, dup: %d, qos: %d, ret: %d, len: %d }") % 
            msg_type % dup_flag % qos % retain % len);
    }    
};

inline std::ostream& operator << (std::ostream& s, const fixed_header& header) {
    s << header.str();
    return s;
}

/**
 * A generic MQTT message.
 *
 * This class provides a way to store polymorphic MQTT messages. It wraps a fixed header and
 * some implementation of a concrete MQTT message. On its constructor, the precise message
 * instance is determined. The fixed header can be used to check what specific message is
 * to be expected to be stored, and the different getters provides `util::option` instances
 * of the different message types.
 */
class message {
public:

    message(const fixed_header& hd, const connect_message& msg)
      : _header(hd), _connect(util::make_some(msg)),
        _content_str(std::bind(&connect_message::str, msg)) {}

    message(const fixed_header& hd, const connect_ack_message& msg)
      : _header(hd), _connect_ack(util::make_some(msg)),
        _content_str(std::bind(&connect_ack_message::str, msg)) {}

    const fixed_header& header() const { return _header; }

    /** Some `connect_message` if it contains a connect message, none otherwise. */
    const util::option<connect_message>& connect() const { return _connect; }

    /** Some `connect_ack_message` if it contains a connect ack message, none otherwise. */
    const util::option<connect_ack_message>& connect_ack() const { return _connect_ack; }

    std::string str() const {
        return util::format("{ header: %s, content: %s}", header().str(), _content_str());
    }

private:

    fixed_header _header;
    util::option<connect_message> _connect;
    util::option<connect_ack_message> _connect_ack;
    std::function<std::string(void)> _content_str;
};

inline std::ostream& operator << (std::ostream& s, const message& msg) {
    s << msg.str();
    return s;
}

using shared_message = std::shared_ptr<message>;

template <class Encoder = codecs::encoder<connect_message>>
shared_message make_connect(const connect_message::client_id& id,
                            const util::option<connect_credentials>& credentials,
                            const util::option<connect_will>& will,
                            unsigned int keep_alive,
                            bool clean_session) {
    connect_message msg = { id, credentials, will, keep_alive, clean_session };
    fixed_header header = {
        message_type::CONNECT,      // msg_type
        false,                      // dup_flag
        qos_level::QOS_0,           // qos
        false,                      // retain
        Encoder::encode_len(msg)    // length
    };
    return std::make_shared<message>(header, msg);
}

template <class Encoder = codecs::encoder<connect_ack_message>>
shared_message make_connect_ack(connect_return_code ret_code) {
    connect_ack_message msg = { ret_code };
    fixed_header header = {
        message_type::CONNACK,      // msg_type
        false,                      // dup_flag
        qos_level::QOS_0,           // qos
        false,                      // retain
        Encoder::encode_len(msg)    // length
    };
    return std::make_shared<message>(header, msg);
}

}}

#endif

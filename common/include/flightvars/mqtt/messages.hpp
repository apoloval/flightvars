/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
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
#include <flightvars/mqtt/qos.hpp>
#include <flightvars/util/exception.hpp>

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

std::string message_type_str(message_type mt) {
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

std::ostream& operator << (std::ostream& s, const message_type& mt) {
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

std::ostream& operator << (std::ostream& s, const fixed_header& header) {
    s << header.str();
    return s;
}

struct message {
    fixed_header header;
    std::unique_ptr<connect_message> connect;

    message(const fixed_header& hd, const connect_message& msg)
      : header(hd), connect(std::make_unique<connect_message>(msg)) {}
};

std::ostream& operator << (std::ostream& s, const message& msg) {
    // TODO: implement this
    s << "message";
    return s;
}

using shared_message = std::shared_ptr<message>;

}}

#endif

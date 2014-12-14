/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_MQTT_CODECS_H
#define FLIGHTVARS_MQTT_CODECS_H

#include <flightvars/mqtt/codecs/connect.hpp>
#include <flightvars/mqtt/codecs/connect_ack.hpp>
#include <flightvars/mqtt/codecs/fixed-header.hpp>
#include <flightvars/mqtt/codecs/types.hpp>

namespace flightvars { namespace mqtt {

/**
 * Encode a MQTT message into the given buffer.
 *
 * The buffer is flipped after encoded bytes are transferred.
 */
void encode(const message& msg, io::buffer& buff) {
    auto header = msg.header();
    codecs::encoder<fixed_header>::encode(header, buff);
    switch (header.msg_type) {
        case message_type::CONNECT:
            codecs::encoder<connect_message>::encode(msg.connect().get(), buff);
            break;
        case message_type::CONNACK:
            codecs::encoder<connect_ack_message>::encode(msg.connect_ack().get(), buff);
            break;
        default:
            throw std::runtime_error(util::format("cannot encode message of unknown type %s",
                message_type_str(header.msg_type)));
    }
    buff.flip();
}

/**
 * Decode a MQTT message from its fixed header and the buffer that contains the message content.
 *
 * The given buffer should be ready to extract the bytes corresponding to the message body.
 */
shared_message decode(const fixed_header& header, io::buffer& buff) {
    switch (header.msg_type) {
        case message_type::CONNECT:
            return std::make_shared<message>(
                header, codecs::decoder<connect_message>::decode(buff));
        case message_type::CONNACK:
            return std::make_shared<message>(
                header, codecs::decoder<connect_ack_message>::decode(buff));
        default:
            throw std::runtime_error(util::format("cannot decode message of unknown type %s",
                message_type_str(header.msg_type)));
    }
}

}}

#endif

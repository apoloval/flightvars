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
#include <flightvars/mqtt/codecs/fixed-header.hpp>
#include <flightvars/mqtt/codecs/types.hpp>

namespace flightvars { namespace mqtt {

/**
 * Decode a MQTT message from its fixed header and the buffer that contains the message content.
 *
 * The given buffer should be ready to extract the bytes corresponding to the message body.
 */
shared_message decode(const fixed_header& header, io::buffer& buff) {
    switch (header.msg_type) {
        case message_type::CONNECT: {
            auto connect = codecs::decoder<connect_message>::decode(buff);
            return std::make_shared<message>(header, connect);
            break;
        }
        default:
            throw std::runtime_error(util::format("cannot decode message of unknown type %s",
                message_type_str(header.msg_type)));
    }
}

}}

#endif

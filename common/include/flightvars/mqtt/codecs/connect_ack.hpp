/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_MQTT_CODECS_CONNECT_ACK_H
#define FLIGHTVARS_MQTT_CODECS_CONNECT_ACK_H

#include <flightvars/mqtt/messages.hpp>
#include <flightvars/mqtt/codecs/types.hpp>
#include <flightvars/util/format.hpp>

namespace flightvars { namespace mqtt { namespace codecs {

template <>
struct decoder<connect_ack_message> {

    using value_type = connect_ack_message;

    static value_type decode(io::buffer& buff) {
        decoder<std::uint8_t>::decode(buff); // first byte is reserved and unused
        auto ret_code = static_cast<connect_return_code>(decoder<std::uint8_t>::decode(buff));
        return connect_ack_message(ret_code);
    }
};

template <>
struct encoder<connect_ack_message> {

    using value_type = connect_ack_message;

    static std::size_t encode_len(const value_type& conn) { return 2;  }

    static void encode(const value_type& conn_ack, io::buffer& buff) {
        encoder<std::uint8_t>::encode(0, buff); // first byte is reserved and unused
        encoder<std::uint8_t>::encode(static_cast<std::uint8_t>(conn_ack.return_code()), buff);
    }
};

}}}

#endif

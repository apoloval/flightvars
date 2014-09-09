/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_MQTT_DECODER_FIXED_HEADER_H
#define FLIGHTVARS_MQTT_DECODER_FIXED_HEADER_H

#include <flightvars/mqtt/messages.hpp>
#include <flightvars/mqtt/decoder/types.hpp>

namespace flightvars { namespace mqtt { namespace decoder {

template <>
struct decoder<fixed_header> {

    using value_type = fixed_header;

    static value_type decode(buffer& buff) {
        fixed_header header;
        auto b1 = buff.safe_read_value<std::uint8_t>();
        
        header.msg_type = static_cast<message_type>(b1 >> 4);
        header.dup_flag = b1 & 0x08;
        header.qos = static_cast<qos_level>((b1 >> 1) & 0x03);
        header.retain = b1 & 0x01;
        header.len = decode_length(buff);

        return header;
    }
    

private:

    static std::size_t decode_length(buffer& buff) {
        std::size_t value = 0;
        for (std::size_t i = 0; i < 4; i++) {
            auto digit = buff.safe_read_value<std::uint8_t>();
            value += std::size_t(digit & 0x7f) << (i * 7);
            if (!(digit & 0x80)) { 
                break; 
            } else if (i == 3) {
                throw decode_error(
                    "cannot decode fixed header length: 4th byte has the continuation bit set");
            }
        }
        return value;
    }    
};

}}}

#endif

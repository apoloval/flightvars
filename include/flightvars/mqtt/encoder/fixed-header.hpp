/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_MQTT_ENCODER_FIXED_HEADER_H
#define FLIGHTVARS_MQTT_ENCODER_FIXED_HEADER_H

#include <flightvars/mqtt/encoder/types.hpp>

namespace flightvars { namespace mqtt { namespace encoder {

template <>
struct encoder<fixed_header> {

    using value_type = fixed_header;

    static void encode(const value_type& fh, buffer& buff) {
        std::uint8_t b1;
        b1 |= static_cast<std::uint8_t>(fh.msg_type) << 4;
        b1 |= fh.dup_flag ? 0x08 : 0x00;
        b1 |= (static_cast<std::uint8_t>(fh.qos) & 0x03) << 1;
        b1 |= fh.retain ? 0x01 : 0x00;
        buff.write_value<std::uint8_t>(b1);

        encode_length(fh, buff);
    }

private:

    static void encode_length(const value_type& fh, buffer& buff) {
        std::size_t value = fh.len;
        for (int i = 0; i < 4; i++) {
            auto digit = value & 0x7f;
            value >>= 7;
            if (value) {
                digit |= 0x80;
                buff.safe_write_value<std::uint8_t>(digit);
                if (i == 3) {
                    throw encode_error(boost::format(
                        "cannot encode fixed header length %d: must be less than 256MB") % fh.len);
                }
            } else {
                buff.safe_write_value<std::uint8_t>(digit);
                break;
            }
        }
    }
};

}}}

#endif

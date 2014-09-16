/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_MQTT_CODECS_TYPES_H
#define FLIGHTVARS_MQTT_CODECS_TYPES_H

#include <cinttypes>

#include <flightvars/util/endian.hpp>
#include <flightvars/util/exception.hpp>

namespace flightvars { namespace mqtt { namespace codecs {

FLIGHTVARS_DECL_EXCEPTION(decode_error);

template <class T>
struct decoder;

template <>
struct decoder<std::uint8_t> {
    using value_type = std::uint8_t;

    static value_type decode(buffer& buff) {
        return buff.safe_read_value<value_type>();
    }
};

template <>
struct decoder<std::uint16_t> {
    using value_type = std::uint16_t;

    static value_type decode(buffer& buff) {
        return from_big_endian(buff.safe_read_value<value_type>());
    }
};


template <>
struct decoder<std::string> {

    using value_type = std::string;

    static value_type decode(buffer& buff) {
        auto len = from_big_endian(buff.safe_read_value<std::uint16_t>());
        auto content = new char[len + 1];
        buff.safe_read(content, len * sizeof(char));
        content[len] = 0;
        std::string result(content);
        delete content;
        return result;
    }
};

FLIGHTVARS_DECL_EXCEPTION(encode_error);

template <class T>
struct encoder;

template <>
struct encoder<std::uint8_t> {
    using value_type = std::uint8_t;

    static void encode(const value_type& num, buffer& buff) {
        buff.safe_write_value(num);
    }
};

template <>
struct encoder<std::uint16_t> {
    using value_type = std::uint16_t;

    static void encode(const value_type& num, buffer& buff) {
        buff.safe_write_value(to_big_endian(num));
    }
};

template <>
struct encoder<std::string> {
    using value_type = std::string;

    static void encode(const value_type& str, buffer& buff) {
        std::uint16_t len = str.size();
        buff.safe_write_value(to_big_endian(len));
        buff.safe_write(str.c_str(), len);
    }
};

}}}

#endif

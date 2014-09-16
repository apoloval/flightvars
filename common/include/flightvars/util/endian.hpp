/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_UTIL_ENDIAN_H
#define FLIGHTVARS_UTIL_ENDIAN_H

#include <cinttypes>

#include <boost/detail/endian.hpp>

namespace flightvars { namespace util {

struct endianness {
#ifdef BOOST_BIG_ENDIAN
    static constexpr bool is_big_endian = true;
    static constexpr bool is_little_endian = false;
#else
    static constexpr bool is_big_endian = false;
    static constexpr bool is_little_endian = true;
#endif

    static std::uint16_t swap(std::uint16_t num) {
        std::uint16_t a = (num & 0xff00) >> 8;
        std::uint16_t b = (num & 0x00ff) << 8;
        return a | b;
    }

    static std::uint32_t swap(std::uint32_t num) {
        std::uint32_t a = (num & 0xff000000) >> 24;
        std::uint32_t b = (num & 0x00ff0000) >> 8;
        std::uint32_t c = (num & 0x0000ff00) << 8;
        std::uint32_t d = (num & 0x000000ff) << 24;
        return a | b | c | d;
    }
};

template <class T>
T to_big_endian(T num) {
    return (endianness::is_big_endian) ? num : endianness::swap(num);    
}

template <class T>
T to_little_endian(T num) {
    return (endianness::is_little_endian) ? num : endianness::swap(num);
}

template <class T>
T from_big_endian(T num) {
    return (endianness::is_big_endian) ? num : endianness::swap(num);    
}

template <class T>
T from_little_endian(T num) {
    return (endianness::is_little_endian) ? num : endianness::swap(num);
}

}}

#endif

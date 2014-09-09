/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_UTIL_FORMAT_H
#define FLIGHTVARS_UTIL_FORMAT_H

#include <boost/format.hpp>

namespace flightvars { namespace util {

std::string format(const boost::format& fmt) {
    return fmt.str();
}

template <class T, class... Args>
std::string format(boost::format& fmt, const T& value, Args... args) {
    return format(fmt % value, args...);
}

template <class T, class... Args>
std::string format(const char* fmt, const T& value, Args... args) {
    boost::format bfmt(fmt);
    return format(bfmt, value, args...);
}

}}

#endif

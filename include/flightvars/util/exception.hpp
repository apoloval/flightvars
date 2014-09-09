/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_UTIL_EXCEPTION_H
#define FLIGHTVARS_UTIL_EXCEPTION_H

#include <exception>
#include <string>

#include <boost/format.hpp>

namespace flightvars { namespace util {

class exception : public std::exception {
public:

    exception(const std::string& msg) : _msg(msg) {}

    const char* what() const throw() override {
        return _msg.c_str();
    }

private:

    std::string _msg;
};

}}

#define FV_DECL_EXCEPTION(classname) \
    class classname : public flightvars::util::exception { \
    public: \
        classname(const std::string& msg) : flightvars::util::exception(msg) {} \
        classname(const boost::format& fmt) : flightvars::util::exception(fmt.str()) {} \
    };

#define FLIGHTVARS_DECL_EXCEPTION(classname) FV_DECL_EXCEPTION(classname)

#endif

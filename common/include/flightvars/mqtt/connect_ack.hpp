/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_MQTT_CONNECT_ACK_H
#define FLIGHTVARS_MQTT_CONNECT_ACK_H

#include <flightvars/util/format.hpp>

namespace flightvars { namespace mqtt {

enum class connect_return_code {
    CONNECTION_ACCEPTED             = 0,
    UNACCEPTABLE_PROTOCOL_VERSION   = 1,
    IDENTIFIER_REJECTED             = 2,
    SERVER_UNAVAILABLE              = 3,
    BAD_USERNAME_OR_PASSWORD        = 4,
    NOT_AUTHORIZED                  = 5,
};

inline std::string connect_return_code_str(connect_return_code rc) {
    switch (rc) {
        case connect_return_code::CONNECTION_ACCEPTED:
            return "CONNECTION_ACCEPTED";
        case connect_return_code::UNACCEPTABLE_PROTOCOL_VERSION:
            return "UNACCEPTABLE_PROTOCOL_VERSION";
        case connect_return_code::IDENTIFIER_REJECTED:
            return "IDENTIFIER_REJECTED";
        case connect_return_code::SERVER_UNAVAILABLE:
            return "SERVER_UNAVAILABLE";
        case connect_return_code::BAD_USERNAME_OR_PASSWORD:
            return "BAD_USERNAME_OR_PASSWORD";
        case connect_return_code::NOT_AUTHORIZED:
            return "NOT_AUTHORIZED";
        default:
            return "UNKNOWN";
    }
}

inline std::ostream& operator << (std::ostream& s, const connect_return_code& rc) {
    s << connect_return_code_str(rc);
    return s;
}

class connect_ack_message {
public:

    connect_ack_message(connect_return_code ret_code) : _return_code(ret_code) {}

    connect_return_code return_code() const { return _return_code; }

    std::string str() const {
        return util::format("{ %s }", connect_return_code_str(return_code()));
    }

private:

    connect_return_code _return_code;
};

inline std::ostream& operator << (std::ostream& s, const connect_ack_message& msg) {
    s << msg.str();
    return s;
}

}}

#endif

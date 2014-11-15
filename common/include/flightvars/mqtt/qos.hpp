/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_MQTT_QOS_H
#define FLIGHTVARS_MQTT_QOS_H

#include <cinttypes>

#include <boost/format.hpp>

#include <flightvars/util/exception.hpp>

namespace flightvars { namespace mqtt {

enum class qos_level {
    QOS_0,
    QOS_1,
    QOS_2,
    QOS_RESERVED_3
};

inline std::string qos_level_str(qos_level qos) {
    switch (qos) {
        case qos_level::QOS_0:          return "QoS-0";
        case qos_level::QOS_1:          return "QoS-1";
        case qos_level::QOS_2:          return "QoS-2";
        case qos_level::QOS_RESERVED_3: return "QoS-reserved";
        default:                        return "QoS-unknown";
    }
}

inline std::ostream& operator << (std::ostream& s, const qos_level& qos) {
    s << qos_level_str(qos);
    return s;
}

}}

#endif

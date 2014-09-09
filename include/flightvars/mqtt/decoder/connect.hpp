/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_MQTT_DECODER_CONNECT_H
#define FLIGHTVARS_MQTT_DECODER_CONNECT_H

#include <flightvars/mqtt/messages.hpp>
#include <flightvars/mqtt/decoder/types.hpp>
#include <flightvars/util/format.hpp>

namespace flightvars { namespace mqtt { namespace decoder {

template <>
struct decoder<connect_message> {

    using value_type = connect_message;

    static value_type decode(buffer& buff) {
        auto proto_name = decoder<std::string>::decode(buff);
        if (proto_name != "MQIsdp") {
            throw decode_error(format("cannot decode connect message: invalid protocol name %s",
                proto_name));
        }

        auto proto_ver = decoder<uint8_t>::decode(buff);
        if (proto_ver != 3) {
            throw decode_error(format("cannot decode connect message: invalid protocol version %d",
                proto_ver));
        }

        auto flags = decoder<uint8_t>::decode(buff);
        auto has_username = flags & 0x80;
        auto has_password = flags & 0x40;
        auto will_retain = flags & 0x20;
        auto will_qos = static_cast<qos_level>((flags >> 3) & 0x03);
        auto has_will = flags & 0x04;
        auto clean_session = flags & 0x02;

        auto keep_alive = decoder<uint16_t>::decode(buff);

        std::string client_id = decoder<std::string>::decode(buff);
        std::string will_topic = has_will ? decoder<std::string>::decode(buff) : "";
        std::string will_message = has_will ? decoder<std::string>::decode(buff) : "";
        std::string username;
        try {
            username = has_username ? decoder<std::string>::decode(buff) : "";
        } catch (const buffer_underflow&) {
            has_username = false;
        }
        std::string password;
        try {
            password = has_password ? decoder<std::string>::decode(buff) : "";
        } catch (const buffer_underflow&) {
            has_password = false;
        }

        if (has_password && !has_username) {
            throw decode_error("cannot decode connect message: "
                "flag password is set, but username is missing");
        }

        option<connect_will> will = has_will ? 
            make_some(connect_will(will_topic, will_message, will_qos, will_retain)) :
            make_none<connect_will>();
        option<connect_credentials> credentials = has_username ?
            make_some(connect_credentials(
                username, 
                has_password ? make_some(password) : make_none<connect_credentials::password>())) :
            make_none<connect_credentials>();

        return connect_message(client_id, credentials, will, keep_alive, clean_session);
    }
};

}}}

#endif

/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_MQTT_ENCODER_CONNECT_H
#define FLIGHTVARS_MQTT_ENCODER_CONNECT_H

#include <flightvars/mqtt/encoder/types.hpp>

namespace flightvars { namespace mqtt { namespace encoder {

template <>
struct encoder<connect_message> {

    using value_type = connect_message;

    static void encode(const value_type& conn, buffer& buff) {
        encoder<std::string>::encode("MQIsdp", buff);
        encoder<std::uint8_t>::encode(3, buff);

        encode_flags(conn, buff);
        encoder<std::uint16_t>::encode(conn.keep_alive(), buff);
        encoder<std::string>::encode(conn.get_client_id(), buff);

        conn.get_will().foreach([&](const connect_will& will) {
            encoder<std::string>::encode(will.get_topic(), buff);
            encoder<std::string>::encode(will.get_message(), buff);
        });
        conn.get_credentials().foreach([&](const connect_credentials& credentials) {
            encoder<std::string>::encode(credentials.get_username(), buff);
            credentials.get_password().foreach([&](const connect_credentials::password& pwd) {
                encoder<std::string>::encode(pwd, buff);
            });
        });
    }

private:

    static void encode_flags(const value_type& conn, buffer& buff) {
        std::uint8_t byte;
        auto has_username = conn.get_credentials().is_defined();
        auto has_password = conn.get_credentials()
            .fmap<connect_credentials::password>([](const connect_credentials& cred) { 
                return cred.get_password(); 
            }).is_defined();
        auto will_retain = conn.get_will()
            .map<bool>([](const connect_will& will) { return will.retain(); })
            .get_or_else(false);
        auto will_qos = conn.get_will()
            .map<qos_level>([](const connect_will& will) { return will.get_qos(); })
            .get_or_else(qos_level::QOS_0);
        auto has_will = conn.get_will().is_defined();
        auto clean_session = conn.clean_session();

        byte |= has_username ? 0x80 : 0x00;
        byte |= has_password ? 0x40 : 0x00;
        byte |= will_retain ? 0x20 : 0x00;
        byte |= (static_cast<std::uint8_t>(will_qos) << 3);
        byte |= has_will ? 0x04 : 0x00;
        byte |= clean_session ? 0x02 : 0x00;

        encoder<std::uint8_t>::encode(byte, buff);
    }
};

}}}

#endif

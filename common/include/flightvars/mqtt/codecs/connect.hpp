/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_MQTT_CODECS_CONNECT_H
#define FLIGHTVARS_MQTT_CODECS_CONNECT_H

#include <flightvars/mqtt/messages.hpp>
#include <flightvars/mqtt/codecs/types.hpp>
#include <flightvars/util/format.hpp>

namespace flightvars { namespace mqtt { namespace codecs {

template <>
struct decoder<connect_message> {

    using value_type = connect_message;

    static value_type decode(io::buffer& buff) {
        auto proto_name = decoder<std::string>::decode(buff);
        if (proto_name != "MQIsdp") {
            throw decode_error(util::format("cannot decode connect message: invalid protocol name %s",
                proto_name));
        }

        auto proto_ver = decoder<uint8_t>::decode(buff);
        if (proto_ver != 3) {
            throw decode_error(util::format("cannot decode connect message: invalid protocol version %d",
                proto_ver));
        }

        auto flags = decoder<uint8_t>::decode(buff);
        auto has_username = (flags & 0x80) > 0;
        auto has_password = (flags & 0x40) > 0;
        auto will_retain = (flags & 0x20) > 0;
        auto will_qos = static_cast<qos_level>((flags >> 3) & 0x03);
        auto has_will = (flags & 0x04) > 0;
        auto clean_session = (flags & 0x02) > 0;

        auto keep_alive = decoder<uint16_t>::decode(buff);

        std::string client_id = decoder<std::string>::decode(buff);
        std::string will_topic = has_will ? decoder<std::string>::decode(buff) : "";
        std::string will_message = has_will ? decoder<std::string>::decode(buff) : "";
        std::string username;
        try {
            username = has_username ? decoder<std::string>::decode(buff) : "";
        } catch (const io::buffer_underflow&) {
            has_username = false;
        }
        std::string password;
        try {
            password = has_password ? decoder<std::string>::decode(buff) : "";
        } catch (const io::buffer_underflow&) {
            has_password = false;
        }

        if (has_password && !has_username) {
            throw decode_error("cannot decode connect message: "
                "flag password is set, but username is missing");
        }

        util::option<connect_will> will = has_will ?
            util::make_some(connect_will(will_topic, will_message, will_qos, will_retain)) :
            util::make_none<connect_will>();
        util::option<connect_credentials> credentials = has_username ?
           util:: make_some(connect_credentials(
                username, 
                has_password ? util::make_some(password)
                    : util::make_none<connect_credentials::password>())) :
            util::make_none<connect_credentials>();

        return connect_message(client_id, credentials, will, keep_alive, clean_session);
    }
};

template <>
struct encoder<connect_message> {

    using value_type = connect_message;

    static std::size_t encode_len(const value_type& conn) {
        using namespace std::placeholders;

        return 12 +
            string_sizeof(conn.get_client_id()) +
            conn.get_will().fold<std::size_t>([](const connect_will& will) {
                return string_sizeof(will.get_topic()) + string_sizeof(will.get_message());
            }, 0).get() +
            conn.get_credentials().fold<std::size_t>([](const connect_credentials& cred) {
                return string_sizeof(cred.get_username()) +
                    cred.get_password().fold<std::size_t>(std::bind(&string_sizeof, _1), 0).get();
            }, 0).get();
    }

    static void encode(const value_type& conn, io::buffer& buff) {
        encoder<std::string>::encode("MQIsdp", buff);
        encoder<std::uint8_t>::encode(3, buff);

        encode_flags(conn, buff);
        encoder<std::uint16_t>::encode(conn.keep_alive(), buff);
        encoder<std::string>::encode(conn.get_client_id(), buff);

        conn.get_will().for_each([&](const connect_will& will) {
            encoder<std::string>::encode(will.get_topic(), buff);
            encoder<std::string>::encode(will.get_message(), buff);
        });
        conn.get_credentials().for_each([&](const connect_credentials& credentials) {
            encoder<std::string>::encode(credentials.get_username(), buff);
            credentials.get_password().for_each([&](const connect_credentials::password& pwd) {
                encoder<std::string>::encode(pwd, buff);
            });
        });
    }

private:

    static std::size_t string_sizeof(const std::string& s) { return 2 + s.length(); }

    static void encode_flags(const value_type& conn, io::buffer& buff) {
        std::uint8_t byte = 0;
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

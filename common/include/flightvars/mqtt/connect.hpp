/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_MQTT_CONNECT_H
#define FLIGHTVARS_MQTT_CONNECT_H

#include <cinttypes>

#include <boost/format.hpp>

#include <flightvars/mqtt/qos.hpp>
#include <flightvars/util/option.hpp>
#include <flightvars/util/exception.hpp>

namespace flightvars { namespace mqtt {

class connect_credentials {
public:

    using username = std::string;
    using password = std::string;

    connect_credentials(const username& usr, const util::option<password>& pwd)
        : _username(usr), _password(pwd) {}

    connect_credentials(const username& usr, const password& pwd)
        : _username(usr), _password(util::make_some(pwd)) {}

    connect_credentials(const username& usr) : _username(usr) {}

    connect_credentials(const connect_credentials& other) : 
        _username(other._username), _password(other._password) {}

    const username& get_username() const { return _username; }

    const util::option<password> get_password() const { return _password; }

private:

    username _username;
    util::option<password> _password;
};

class connect_will {
public:

    using topic = std::string;
    using message = std::string;

    connect_will(const topic& t, const message& m, const qos_level& qos, bool retain)
        : _topic(t), _message(m), _qos(qos), _retain(retain) {}

    connect_will(const connect_will& other) :
        _topic(other._topic), _message(other._message), _qos(other._qos), _retain(other._retain) {}

    const topic& get_topic() const { return _topic; }

    const message& get_message() const { return _message; }

    const qos_level& get_qos() const { return _qos; }

    bool retain() const { return _retain; }

private:

    topic _topic;
    message _message;
    qos_level _qos;
    bool _retain;
};

class connect_message {
public:

    using client_id = std::string;

    connect_message(const client_id& id, 
                    const util::option<connect_credentials>& credentials,
                    const util::option<connect_will>& will,
                    unsigned int keep_alive, 
                    bool clean_session) :
        _id(id), _will(will), _credentials(credentials), 
        _keep_alive(keep_alive), _clean_session(clean_session) {}

    connect_message(const client_id& id, 
                    const connect_credentials& credentials, 
                    const connect_will& will, 
                    unsigned int keep_alive, 
                    bool clean_session) :
        _id(id), _will(util::make_some(will)), _credentials(util::make_some(credentials)),
        _keep_alive(keep_alive), _clean_session(clean_session) {}

    connect_message(const client_id& id, 
                    const connect_credentials& credentials, 
                    unsigned int keep_alive, 
                    bool clean_session) :
        _id(id), _credentials(util::make_some(credentials)),
        _keep_alive(keep_alive), _clean_session(clean_session) {}

    connect_message(const client_id& id, 
                    const connect_will& will, 
                    unsigned int keep_alive, 
                    bool clean_session) :
        _id(id), _will(util::make_some(will)), _keep_alive(keep_alive),
        _clean_session(clean_session) {}

    connect_message(const client_id& id, 
                    unsigned int keep_alive, 
                    bool clean_session) : 
        _id(id), _keep_alive(keep_alive), _clean_session(clean_session) {}

    connect_message(const connect_message& other) :
        _id(other._id), _will(other._will), _credentials(other._credentials), 
        _keep_alive(other._keep_alive), _clean_session(other._clean_session) {}

    const client_id& get_client_id() const { return _id; }

    const util::option<connect_credentials>& get_credentials() const { return _credentials; }

    const util::option<connect_will>& get_will() const { return _will; }

    unsigned int keep_alive() const { return _keep_alive; }

    bool clean_session() const { return _clean_session; }

private:

    client_id _id;
    util::option<connect_will> _will;
    util::option<connect_credentials> _credentials;
    unsigned int _keep_alive;
    bool _clean_session;
};

}}

#endif

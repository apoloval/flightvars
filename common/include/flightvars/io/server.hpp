/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_IO_SERVER_H
#define FLIGHTVARS_IO_SERVER_H

namespace flightvars { namespace io {

/**
 * An type trait that determines whether a given type T is an IO server.
 *
 * An IO server is any object that is able to server IO connections using an specific transport.
 * Any type that claims to be a server must fit the following requirements:
 *
 *  - It must define an inner type `connection_type`, which determines the type of the connections
 *    managed by the server. This can be any arbitrary type as long as it fits to Connection
 *    concept determined by `is_connection` type trait.
 *
 *  - It must provide an `accept()` function with the following signature:
 *
 *      future<connection_type> accept()
 *
 *    This function would return a future with the next connection that will be established by
 *    the server, or an error if something went wrong. The returned future can be composed using
 *    `next()`, `then()` and `finally()` function members to set a behavior on new connections.
 */
template <class T>
struct is_server {
    static constexpr bool value = false;
};

}}

#endif

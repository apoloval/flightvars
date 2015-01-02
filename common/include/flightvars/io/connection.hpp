/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_IO_CONNECTION_H
#define FLIGHTVARS_IO_CONNECTION_H

#include <flightvars/concurrent/future.hpp>
#include <flightvars/io/buffer.hpp>

namespace flightvars { namespace io {

template <class T>
struct is_connection {

    // TODO:
    // Requirement: it should have a `shared_ptr` type of type `std::shared_ptr<T>`

    // TODO:
    // Requirement: it should provide a way to be written to a `std::ostream`

    static constexpr bool value = true;
};

template <class Connection>
concurrent::future<std::size_t>
read_remaining(Connection& conn, buffer& buff) {
    return conn.read(buff, buff.remaining());
}

template <class Connection>
concurrent::future<std::size_t>
write_remaining(Connection& conn, buffer& buff) {
    return conn.write(buff, buff.remaining());
} 

}}

#endif

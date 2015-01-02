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

/**
 * An type trait that determines whether a given type T is an IO connection.
 *
 * Any type that claims to be a connection must fit the following requirements:
 *
 *  - It must provide a `read()` function with the following signature:
 *
 *      future<std::size_t> read(buffer& buff, std::size_t bytes)
 *
 *    This function would read bytes from the connection and will store them in the
 *    buffer passed as argument. It won't read more bytes than specified by `bytes`
 *    argument. As result, it produces a future indicating the number of bytes that
 *    were successfully read, or an error if something went wrong.
 *
 *  - It must provide a `write()` function with the following signature:
 *
 *      future<std::size_t> write(buffer& buff, std::size_t bytes)
 *
 *    This function would write bytes from the buffer passed as argument to the connection
 *    It won't write more bytes than specified by `bytes` argument. As result, it
 *    produces a future indicating the number of bytes that were successfully written,
 *    or an error if something went wrong.
 *
 *  - It must provide a `close()` function with the following signature:
 *
 *      void close()
 *
 *    This function closes the connection to the other peer.
 */
template <class T>
struct is_connection {
    static constexpr bool value = false;
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

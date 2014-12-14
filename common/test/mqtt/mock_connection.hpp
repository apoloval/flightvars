/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_IO_MOCK_CONNECTION_H
#define FLIGHTVARS_IO_MOCK_CONNECTION_H

namespace flightvars { namespace mqtt {

struct mock_connection {
    using shared_ptr = std::shared_ptr<mock_connection>;
    io::buffer read_buffer;
    io::buffer write_buffer;

    concurrent::future<io::shared_buffer> read(const io::shared_buffer& buff, std::size_t bytes) {
        BOOST_CHECK_EQUAL(bytes, buff->write(read_buffer, bytes));
        read_buffer.inc_pos(bytes);
        return concurrent::make_future_success(buff);
    }

    template <class Message>
    void prepare_read(const fixed_header& hd, const Message& msg) {
        read_buffer.reset();
        codecs::encoder<fixed_header>::encode(hd, read_buffer);
        codecs::encoder<Message>::encode(msg, read_buffer);
        read_buffer.flip();
    }
};

std::ostream& operator << (std::ostream& s, const mock_connection& conn) {
    s << "mock connection";
    return s;
}

mock_connection::shared_ptr make_mock_connection() {
    return std::make_shared<mock_connection>();
}

}}

#endif

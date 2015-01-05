/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_IO_MOCK_CONNECTION_H
#define FLIGHTVARS_IO_MOCK_CONNECTION_H

#include <flightvars/concurrent/future.hpp>
#include <flightvars/io/connection.hpp>
#include <flightvars/mqtt/codecs.hpp>
#include <flightvars/mqtt/messages.hpp>

namespace flightvars { namespace mqtt {

FV_DECL_EXCEPTION(mock_connection_closed);

class mock_connection {
public:

    using shared_ptr = std::shared_ptr<mock_connection>;    

    concurrent::future<std::size_t> read(io::buffer& buff, std::size_t bytes) {
        if (_read_buffer.remaining() == 0) {
            return concurrent::make_future_failure<std::size_t>(
                mock_connection_closed("mock connection is closed"));
        }
        BOOST_CHECK_EQUAL(bytes, buff.write(_read_buffer, bytes));
        _read_buffer.inc_pos(bytes);
        return concurrent::make_future_success<std::size_t>(bytes);
    }

    concurrent::future<std::size_t>
    write(io::buffer& buff, std::size_t bytes) {
        _write_buffer.write(buff, bytes);
        return concurrent::make_future_success<std::size_t>(bytes);
    }

    void prepare_read_message(const message& msg) {
        _read_buffer.reset();
        encode(msg, _read_buffer);
        _read_buffer.flip();
    }

    void prepare_read_messages(std::initializer_list<message> messages) {
        _read_buffer.reset();
        std::for_each(messages.begin(), messages.end(), [this](const message& msg) {
            encode(msg, _read_buffer);
        });
        _read_buffer.flip();
    }

    message written_message() {
        _write_buffer.flip();
        auto header = codecs::decoder<fixed_header>::decode(_write_buffer);
        _write_buffer.reset();
        return decode(header, _write_buffer);
    }

private:

    io::buffer _read_buffer;
    io::buffer _write_buffer;
};

inline std::ostream& operator << (std::ostream& s, const mock_connection& conn) {
    s << "mock connection";
    return s;
}

inline mock_connection::shared_ptr make_mock_connection() {
    return std::make_shared<mock_connection>();
}

}}

template <>
struct flightvars::io::is_connection<flightvars::mqtt::mock_connection> {
    static constexpr bool value = true;
};

#endif

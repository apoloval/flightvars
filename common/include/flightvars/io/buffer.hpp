/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_IO_BUFFER_H
#define FLIGHTVARS_IO_BUFFER_H

#define FLIGHTVARS_DEFAULT_BUFFER_SIZE (64*1024) // 64KB

#include <cstddef>
#include <cstdint>

#include <boost/asio.hpp>

#include <flightvars/util/exception.hpp>
#include <flightvars/util/option.hpp>

namespace flightvars { namespace io {

FLIGHTVARS_DECL_EXCEPTION(buffer_overflow);
FLIGHTVARS_DECL_EXCEPTION(buffer_underflow);

class buffer {
public:

    buffer(std::size_t size = FLIGHTVARS_DEFAULT_BUFFER_SIZE) 
        : _size(size), 
          _limit(size),
          _pos(0),
          _data(new std::uint8_t[size]) {}

    ~buffer() { delete[] _data; }

    buffer(std::initializer_list<std::uint8_t> bytes) : buffer(bytes.size()) {
        write(bytes.begin(), bytes.size());
        flip();
    }

    buffer(buffer&& other) 
        : _size(other._size), _limit(other._limit), _pos(other._pos), _data(other._data) {
        other._size = other._limit = other._pos = 0;
        other._data = nullptr;
    }

    std::size_t size() const { return _size; }

    std::size_t limit() const { return _limit; }

    std::size_t pos() const { return _pos; }

    std::size_t set_pos(std::size_t new_pos) const { 
        _pos = std::min(new_pos, _limit); 
        return _pos;
    }

    std::size_t inc_pos(std::size_t inc) const { 
        return set_pos(_pos + inc); 
    }

    std::size_t dec_pos(std::size_t inc) const { 
        if (inc > _pos) { 
            inc = _pos; 
        }
        return set_pos(_pos - inc); 
    }

    std::size_t skip(std::size_t bytes) const {
        return inc_pos(bytes);
    }

    std::size_t remaining() const { return _limit - _pos; }

    void* data() { return _data + _pos; }

    const void* data() const { return _data + _pos; }

    std::uint8_t first() const { return *(_data + _pos); }

    std::uint8_t last() const { 
        if (_limit == 0) {
            throw buffer_overflow("cannot obtain last element of buffer when limit is zero");
        }
        return *(_data + _limit - 1); 
    }

    util::option<std::uint8_t> last_opt() const { 
        if (_limit == 0) {
            return make_none<std::uint8_t>();
        }
        return make_some(*(_data + _limit - 1)); 
    }

    boost::asio::mutable_buffer to_boost_asio(std::size_t bytes) {
        return boost::asio::mutable_buffer(data(), std::min(remaining(), bytes));
    }

    boost::asio::mutable_buffer to_boost_asio() {
        return to_boost_asio(remaining());
    }

    boost::asio::const_buffer to_boost_asio(std::size_t bytes) const {
        return boost::asio::const_buffer(data(), std::min(remaining(), bytes));
    }

    boost::asio::const_buffer to_boost_asio() const {
        return to_boost_asio(remaining());
    }

    void reset(bool reset_pos = true) { 
        _limit = _size;
        if (reset_pos) { _pos = 0; }
    }

    void flip() {
        _limit = _pos;
        _pos = 0;
    }

    std::size_t write(const void* from, std::size_t nbytes) {
        auto to_write = std::min(remaining(), nbytes);
        std::memcpy(data(), from, to_write);
        _pos += to_write;
        return to_write;
    }

    void safe_write(const void* from, std::size_t nbytes) {
        if (write(from, nbytes) != nbytes) {
            throw buffer_overflow("buffer overflow while writting bytes");
        }
    }

    std::size_t write(const std::string& data) {
        return write(data.c_str(), data.length());
    }

    std::size_t write(const buffer& other) {
        return write(other.data(), other.remaining());
    }

    std::size_t write(const buffer& other, std::size_t nbytes) {
        return write(other.data(), nbytes);
    }

    template <class T>
    std::size_t write_value(const T& from) {
        return write(&from, sizeof(T));
    }

    template <class T>
    void safe_write_value(const T& from) {
        if (write_value(from) != sizeof(T)) {
            throw buffer_overflow("buffer overflow while writing a value");
        }
    }

    std::size_t read(void* dest, std::size_t nbytes) {
        auto to_read = std::min(remaining(), nbytes);
        std::memcpy(dest, data(), to_read);
        _pos += to_read;
        return to_read;
    }

    void safe_read(void* dest, std::size_t nbytes) {
        if (read(dest, nbytes) != nbytes) {
            throw buffer_underflow("buffer underflow while reading bytes");
        }
    }

    template <class T>
    std::size_t read_value(T& dest) {
        return read(&dest, sizeof(T));
    }

    template <class T>
    T safe_read_value() {
        T value;
        if (read_value(value) != sizeof(T)) {
            throw buffer_underflow("buffer underflow while reading a value");
        }
        return value;
    }
    
    std::size_t read_string(std::string& str, std::size_t len) {
        auto data = new char[len + 1];
        auto nread = read(data, len);
        data[nread] = 0;
        str = data;
        delete data;
        return nread;
    }

    std::string safe_read_string(std::size_t len) {
        std::string str;
        if (read_string(str, len) != len) {
            throw buffer_underflow("buffer underflow while reading a string");
        }
        return str;
    }

    std::size_t read(buffer& other) {
        return read(other.data(), other.remaining());
    }

    std::size_t read(buffer& other, std::size_t nbytes) {
        return read(other.data(), nbytes);
    }

private:

    std::uint8_t* _data;
    mutable std::size_t _pos;
    std::size_t _limit;
    std::size_t _size;
};

std::ostream& operator << (std::ostream& s, const buffer& buff) {
    s << "buffer { size:" << buff.size() <<
        ", limit:" << buff.limit() <<
        ", pos:" << buff.pos() <<
        " }";
    return s;
}

using shared_buffer = std::shared_ptr<buffer>;
using shared_const_buffer = std::shared_ptr<const buffer>;

shared_buffer make_shared_buffer(
        std::size_t size = FLIGHTVARS_DEFAULT_BUFFER_SIZE) {
    return std::make_shared<buffer>(size);
}

shared_buffer make_shared_buffer(
        const std::string& data) {
    auto buff = std::make_shared<buffer>(data.length());
    buff->write(data.c_str(), data.length());
    return buff;
}

}}

#endif

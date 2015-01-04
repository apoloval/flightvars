/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_UTIL_NONCOPYABLE_H
#define FLIGHTVARS_UTIL_NONCOPYABLE_H

namespace flightvars { namespace util {

template <class T>
class noncopyable {
public:

    noncopyable(T&& value) : _value(std::move(value)) {}

    noncopyable(noncopyable&& other) : _value(std::move(other._value)) {}

    noncopyable(const noncopyable&) = delete;

    noncopyable& operator = (noncopyable&& other) {
        _value = std::move(other._value);
        return *this;
    }

    noncopyable& operator = (const noncopyable&) = delete;

    operator const T&() const { return _value; }

    const T& operator* () const { return _value; }

    const T& get() const { return _value; }

private:

    T _value;
};

template <class T>
noncopyable<typename std::decay<T>::type> make_noncopyable(T&& value) {
    return noncopyable<typename std::decay<T>::type>(std::forward<T>(value));
}

}}

#endif

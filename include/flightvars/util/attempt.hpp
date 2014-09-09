/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_UTIL_ATTEMPT_H
#define FLIGHTVARS_UTIL_ATTEMPT_H

#include <exception>
#include <memory>

#include <flightvars/util/exception.hpp>
#include <flightvars/util/option.hpp>

namespace flightvars { namespace util {

/** 
 * An attempt to produce a value of class T. 
 * 
 * This class wraps either a value of class T, or an error encapsulated in a
 * std::exception_ptr value. It may be used to represent an attempt to perform
 * a computation.
 */
template <class T>
class attempt {
public:

    static_assert(std::is_void<T>::value || std::is_copy_constructible<T>::value, 
        "attempt cannot be instantiated with non-copy constructible types");

    /** Create a successful attempt for T != void. */
    template <class U = T>    
    attempt(typename std::enable_if<!std::is_void<U>::value, const U&>::type value) : 
        _value(value) {}

    /** Create a successful attempt for T == void. */
    template <class U = T>    
    attempt(typename std::enable_if<std::is_void<U>::value>::type* value = 0) : _value(true) {}

    /** Create a failure attempt. */
    attempt(const std::exception_ptr& error) : _error(error) {}

    /** Create a failure attempt. */
    template <class E>
    attempt(const E& error) : _error(std::make_exception_ptr(error)) {}

    /** Copy constructor. */
    attempt(const attempt& other) : 
        _value(other._value), _error(other._error) {}

    /** Move constructor. */
    attempt(attempt&& other) : _value(std::move(other._value)), _error(std::move(other._error)) {}

    /** Copy operator. */
    attempt& operator = (const attempt& other) {
        _value = other._value;
        _error = other._error;
        return *this;
    }

    /** Return true if the attempt was successful, false otherwise. */
    bool is_success() const { return _value.is_defined(); }

    /** Return true of the attempt has failed, false otherwise. */
    bool is_failure() const { return !is_success(); }

    /** Return the computed value if success, or throw the error otherwise. */
    template <class U = T>
    typename std::enable_if<!std::is_void<U>::value, const U&>::type
    get() const {
        if (is_success()) { return _value.get(); }
        else { std::rethrow_exception(_error); }
    }

    /** Return the computed value if success, or throw the error otherwise. */
    template <class U = T>
    typename std::enable_if<std::is_void<U>::value>::type
    get() const { if (!is_success()) { std::rethrow_exception(_error); } }

    /** Return the computed value as an option. */
    const option<T>& get_opt() const {
        return _value;
    }

private:

    option<T> _value;
    std::exception_ptr _error;
};

template <class T, class U = attempt<T>>
typename std::enable_if<!std::is_void<T>::value, U>::type 
make_success(const T& value) {
    return attempt<T>(value); 
}

template <class T, class U = attempt<T>>
typename std::enable_if<std::is_void<T>::value, U>::type 
make_success() {
    return attempt<T>(); 
}

template <class T>
attempt<T> make_failure(const std::exception_ptr& error) {
    return attempt<T>(error);
}

template <class T, class E>
attempt<T> make_failure(const E& error) {
    return attempt<T>(error);
}

}}

#endif

/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_CONCURRENT_PROMISE_H
#define FLIGHTVARS_CONCURRENT_PROMISE_H

#include <flightvars/concurrent/new/future.hpp>
#include <flightvars/concurrent/new/shared_state.hpp>
#include <flightvars/util/exception.hpp>
#include <flightvars/util/option.hpp>

namespace flightvars { namespace concurrent { namespace newwave {

FV_DECL_EXCEPTION(bad_promise);
FV_DECL_EXCEPTION(future_already_retrieved);

template <class T>
class promise {
public:

    static_assert(std::is_void<T>::value || std::is_move_constructible<T>::value,
        "cannot instantiate a promise with a non move-constructible type");

    promise() : _future(_state) {}

    promise(promise&&) = default;
    promise(const promise&) = delete;

    bool valid() const { return _state.valid(); }

    future<T> get_future() {
        if (!_future.valid()) {
            throw future_already_retrieved("cannot obtain a future from a promise twice");
        }
        return std::move(_future);
    }

    template <class U = T>
    typename std::enable_if<!std::is_void<U>::value>::type
    set_value(const U& value) {
        check_valid();
        _state.push_success(value);
        _state.reset();
    }

    template <class U = T>
    typename std::enable_if<!std::is_void<U>::value>::type
    set_value(U&& value) {
        check_valid();
        _state.push_success(std::move(value));
        _state.reset();
    }

    template <class U = T>
    typename std::enable_if<std::is_void<U>::value>::type
    set_value(const U* = 0) {
        check_valid();
        _state.push_success();
        _state.reset();
    }

    template <class Exception>
    void set_exception(const Exception& e) {
        check_valid();
        _state.push_failure(e);
        _state.reset();
    }

private:

    shared_state<T> _state;
    future<T> _future;

    void check_valid() {
        if (!_state.valid()) {
            throw bad_promise("invalid operation on invalid promise");
        }
    }
};

}}}

#endif

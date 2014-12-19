/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_CONCURRENT_SHARED_STATE_H
#define FLIGHTVARS_CONCURRENT_SHARED_STATE_H

#include <flightvars/util/attempt.hpp>
#include <flightvars/util/exception.hpp>
#include <flightvars/util/option.hpp>

namespace flightvars { namespace concurrent { namespace newwave {

FV_DECL_EXCEPTION(bad_shared_state);

template <class T>
class shared_state {
public:

    shared_state() : _state(std::make_shared<state>()) {}
    shared_state(const shared_state&) = default;
    shared_state(shared_state&&) = default;

    shared_state& operator = (const shared_state&) = default;
    shared_state& operator = (shared_state&&) = default;

    bool valid() const { return !!_state; }

    void reset() {
        _state = nullptr;
    }

    template <class F>
    void set_push_handler(F&& f) {
        check_valid();
        _state->_push_handler = f;
        if (_state->_retained.is_defined()) {
            f(_state->_retained.extract());
        }
    }

    void clear_push_handler() {
        check_valid();
        _state->_push_handler = nullptr;
    }

    void push(util::attempt<T>&& value) {
        check_valid();
        if (!!_state->_push_handler) {
            _state->_push_handler(std::move(value));
        } else {
            _state->_retained = std::move(value);
        }
    }

    template <class U = T>
    typename std::enable_if<!std::is_void<U>::value>::type
    push_success(const U& value) { push(util::make_success<T>(value)); }

    template <class U = T>
    typename std::enable_if<!std::is_void<U>::value>::type
    push_success(U&& value) { push(util::make_success<T>(std::move(value))); }

    template <class U = T>
    typename std::enable_if<std::is_void<U>::value>::type
    push_success() { push(util::make_success<T>()); }

    template <class Error>
    void push_failure(Error&& error) { push(util::make_failure<T>(std::move(error))); }

private:

    struct state {
        util::option<util::attempt<T>> _retained;
        std::function<void(util::attempt<T>)> _push_handler;
    };

    std::shared_ptr<state> _state;

    void check_valid() {
        if (!valid()) {
            throw bad_shared_state("shared state is not valid");
        }
    }
};

}}}

#endif

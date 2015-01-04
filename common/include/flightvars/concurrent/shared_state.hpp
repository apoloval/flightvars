/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_CONCURRENT_SHARED_STATE_H
#define FLIGHTVARS_CONCURRENT_SHARED_STATE_H

#include <mutex>

#include <flightvars/util/attempt.hpp>
#include <flightvars/util/exception.hpp>
#include <flightvars/util/option.hpp>

namespace flightvars { namespace concurrent {

FV_DECL_EXCEPTION(bad_shared_state);

template <class T>
class shared_state {
public:

    shared_state() : _control(std::make_shared<control_block>()) {}
    shared_state(const shared_state&) = default;
    shared_state(shared_state&&) = default;

    shared_state& operator = (const shared_state&) = default;
    shared_state& operator = (shared_state&&) = default;

    bool valid() const { return !!_control; }

    void reset() {
        _control = nullptr;
    }

    template <class Func>
    void set_push_handler(Func&& f) {
        check_valid();
        _control->set_push_handler(std::forward<Func>(f));
    }

    void clear_push_handler() {
        check_valid();
        _control->clear_push_handler();
    }

    void push(util::attempt<T>&& value) {
        check_valid();
        _control->push(std::move(value));
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

    class control_block {
    public:

        template <class F>
        void set_push_handler(F&& f) {
            std::unique_lock<std::recursive_mutex> lock(_mutex);
            _push_handler = f;
            if (_retained.is_defined()) {
                f(_retained.extract());
            }
        }

        void clear_push_handler() {
            std::unique_lock<std::recursive_mutex> lock(_mutex);
            _push_handler = nullptr;
        }

        void push(util::attempt<T>&& value) {
            std::unique_lock<std::recursive_mutex> lock(_mutex);
            if (!!_push_handler) {
                _push_handler(std::move(value));
            } else {
                _retained = std::move(value);
            }
        }

    private:
        mutable std::recursive_mutex _mutex;
        util::option<util::attempt<T>> _retained;
        std::function<void(util::attempt<T>)> _push_handler;
    };

    std::shared_ptr<control_block> _control;

    void check_valid() {
        if (!valid()) {
            throw bad_shared_state("shared state is not valid");
        }
    }
};

}}

#endif

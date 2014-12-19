/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_CONCURRENT_FUTURE_H
#define FLIGHTVARS_CONCURRENT_FUTURE_H

#include <condition_variable>
#include <mutex>

#include <flightvars/concurrent/new/shared_state.hpp>

namespace flightvars { namespace concurrent { namespace newwave {

FV_DECL_EXCEPTION(bad_future);
FV_DECL_EXCEPTION(future_timeout);

template <class T>
class future {
public:

    static_assert(std::is_void<T>::value || std::is_move_constructible<T>::value,
        "cannot instantiate a future with a non move-constructible type");

    future() { _state.reset(); }

    future(future&& other) : _state(std::move(other._state)) {
        reset_push_handler();
    }

    future(const future&) = delete;

    ~future() {
        clear_push_handler();
    }

    future& operator = (future&& other) {
        clear_push_handler();
        _state = std::move(other._state);
        reset_push_handler();
        return *this;
    }

    bool valid() const { return _state.valid(); }

    bool is_completed() const {
        std::lock_guard<std::recursive_mutex> lock(_mutex);
        return _result.valid();
    }

    template <class U = T>
    typename std::enable_if<!std::is_void<U>::value, U>::type
    get() {
        std::lock_guard<std::recursive_mutex> lock(_mutex);
        wait();
        check_valid(); // someone else could be getting, so it could be invalid after wait
        return _result.extract();
    }

    template <class U = T>
    typename std::enable_if<std::is_void<U>::value, U>::type
    get() {
        wait();
        _result.get();
    }

    void wait() const {
        std::unique_lock<std::recursive_mutex> lock(_mutex);
        check_valid();
        auto completed = std::bind(&future::is_completed, this);
        if (!completed()) {
            _completion_cond.wait(lock, completed);
        }
    }

    template <class R, class P>
    void wait_for(const std::chrono::duration<R,P>& timeout) const {
        std::unique_lock<std::recursive_mutex> lock(_mutex);
        check_valid();
        auto completed = std::bind(&future::is_completed, this);
        if (!completed()) {
            if (!_completion_cond.wait_for(lock, timeout, completed)) {
                throw future_timeout(
                    "future timeout while waiting for completion");
            }
        }
    }

private:

    template <class U>
    friend class promise;

    shared_state<T> _state;
    util::attempt<T> _result;
    mutable std::recursive_mutex _mutex;
    mutable std::condition_variable_any _completion_cond;

    future(const shared_state<T>& state) : _state(state) {
        _state.set_push_handler(std::bind(&future::result_handler, this, std::placeholders::_1));
    }

    void result_handler(util::attempt<T> result) {
        std::lock_guard<std::recursive_mutex> lock(_mutex);
        _result = std::move(result);
        _completion_cond.notify_all();
    }

    void reset_push_handler() {
        _state.set_push_handler(std::bind(&future::result_handler, this, std::placeholders::_1));
    }

    void clear_push_handler() {
        if (_state.valid()) {
            _state.clear_push_handler();
        }
    }

    void check_valid() const {
        if (!valid()) {
            throw bad_future("operation not allowed on not valid future");
        }
    }

    void reset() { _state.reset(); }
};

}}}

#endif

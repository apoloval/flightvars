/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_CONCURRENT_FUTURE_H
#define FLIGHTVARS_CONCURRENT_FUTURE_H

#include <iostream> // TODO: remove this
#include <condition_variable>
#include <mutex>

#include <flightvars/concurrent/executor.hpp>
#include <flightvars/concurrent/shared_state.hpp>

namespace flightvars { namespace concurrent {

FV_DECL_EXCEPTION(bad_future);
FV_DECL_EXCEPTION(future_timeout);

template <class T>
class future {
public:

    static_assert(std::is_void<T>::value || std::is_move_constructible<T>::value,
        "cannot instantiate a future with a non move-constructible type");

    future() { reset_state(); }

    future(future&& other) : _state(std::move(other._state)),
                             _result(std::move(other._result)) {
        reset_push_handler();
    }

    future(const future&) = delete;

    ~future() {
        clear_push_handler();
    }

    future& operator = (future&& other) {
        clear_push_handler();
        _state = std::move(other._state);
        _result = std::move(other._result);
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

    template <class R, class P, class U = T>
    typename std::enable_if<!std::is_void<U>::value, U>::type
    get_for(const std::chrono::duration<R,P>& timeout) {
        wait_for(timeout);
        return get();
    }

    template <class R, class P, class U = T>
    typename std::enable_if<std::is_void<U>::value, U>::type
    get_for(const std::chrono::duration<R,P>& timeout) {
        wait_for(timeout);
        get();
    }

    template <class Func, class Executor = same_thread_executor>
    future<typename std::result_of<Func(T)>::type>
    then(Func&& func, Executor&& exec = Executor()) {
        using U = typename std::result_of<Func(T)>::type;
        auto p = std::make_shared<promise<U>>();
        set_push_handler([p, func](util::attempt<T> result) mutable {
            auto mapped = result.map(func);
            p->set(std::move(mapped));
        }, exec);
        reset_state();
        return p->get_future();
    }

    template <class U, class Func, class Executor = same_thread_executor>
    future<U> next(Func&& func, Executor&& exec = Executor()) {
        auto p = std::make_shared<promise<U>>();
        set_push_handler([p, func, exec](util::attempt<T> result) mutable {
            try {
                auto f = func(result.extract());
                f.finally([p, exec](util::attempt<U> other_result) {
                    p->set(std::move(other_result));
                }, exec);
            }
            catch (...) { p->set_exception(std::current_exception()); }
        }, exec);
        reset_state();
        return p->get_future();
    }

    template <class Func, class Executor = same_thread_executor>
    void finally(Func&& f, Executor&& exec = Executor()) {
        std::unique_lock<std::recursive_mutex> lock(_mutex);
        check_valid();
        set_push_handler(f, exec);
        reset_state();
    }

private:

    template <class U>
    friend class promise;

    shared_state<T> _state;
    util::attempt<T> _result;
    mutable std::recursive_mutex _mutex;
    mutable std::condition_variable_any _completion_cond;

    future(const shared_state<T>& state) : _state(state) {
        reset_push_handler();
    }

    void reset_state() { _state.reset(); }

    void result_handler(util::attempt<T> result) {
        std::lock_guard<std::recursive_mutex> lock(_mutex);
        _result = std::move(result);
        _completion_cond.notify_all();
    }

    void reset_push_handler() {
        set_push_handler(
            std::bind(&future::result_handler, this, std::placeholders::_1),
            same_thread_executor());
    }

    template <class Func, class Executor>
    void set_push_handler(Func&& handler, Executor&& exec) {
        if (is_completed()) {
            run(exec, handler, std::move(_result));
        } else {
            _state.set_push_handler([=](util::attempt<T> result) mutable {
                run(exec, handler, std::move(result));
            });
        }
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

}}

#include <flightvars/concurrent/promise.hpp>

namespace flightvars { namespace concurrent {

template <class T>
typename std::enable_if<!std::is_void<T>::value, future<T>>::type
make_future_success(const T& value) {
    promise<T> p;
    auto f = p.get_future();
    p.set_value(value);
    return f;
}

template <class T>
typename std::enable_if<!std::is_void<T>::value, future<T>>::type
make_future_success(T&& value) {
    promise<T> p;
    auto f = p.get_future();
    p.set_value(std::move(value));
    return f;
}

template <class T>
typename std::enable_if<std::is_void<T>::value, future<T>>::type
make_future_success() {
    promise<T> p;
    auto f = p.get_future();
    p.set_value();
    return f;
}

template <class T, class E>
future<T> make_future_failure(E&& error) {
    promise<T> p;
    auto f = p.get_future();
    p.set_failure(error);
    return f;
}

}}

#endif

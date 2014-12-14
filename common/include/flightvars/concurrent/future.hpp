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
#include <list>

#include <flightvars/concurrent/executor.hpp>
#include <flightvars/concurrent/promise.hpp>
#include <flightvars/util/attempt.hpp> 
#include <flightvars/util/option.hpp>

namespace flightvars { namespace concurrent {

FV_DECL_EXCEPTION(bad_future);
FV_DECL_EXCEPTION(future_timeout);
FV_DECL_EXCEPTION(uncompleted_future);

template <class T>
class future {
public:

    static_assert(std::is_void<T>::value || std::is_copy_constructible<T>::value, 
        "cannot instantiate a future with a non-copy constructible type");

    using listener = std::function<void(const attempt<T>&)>;

    future(promise<T>& p) : _core(new core()) {
        p.add_listener(
            std::bind(&core::on_complete, _core, std::placeholders::_1));
    }

    ~future() {
    }

    future(const future& other) : _core(other._core) {}

    bool is_completed() const { return _core->is_completed(); }

    template <class U = T>
    typename std::enable_if<!std::is_void<U>::value, const U&>::type
    get() const { return _core->get(); }

    template <class U = T>
    typename std::enable_if<std::is_void<U>::value>::type
    get() const { return _core->get(); }

    template <class Executor = concurrent::same_thread_executor>
    void add_listener(const listener& l, const Executor& exec = Executor()) {
        _core->add_listener(std::bind(
            concurrent::run<Executor, listener, attempt<T>>,
            exec, l, std::placeholders::_1));
    }

    template <class U, class Func, class Executor = concurrent::same_thread_executor>
    future<U> map(Func map, const Executor& exec = Executor()) {
        auto p = std::make_shared<promise<U>>();
        auto f = make_future(*p);
        add_listener([p, map](const attempt<T>& result) {
            try {                
                consolidate_map(p, result, map);
            } catch(...) {
                p->set_failure(std::current_exception());
            }
        }, exec);
        return f; 
    }

    template <class U, class Func, class Executor = concurrent::same_thread_executor>
    future<U> fmap(Func map, const Executor& exec = Executor()) {
        auto p = std::make_shared<promise<U>>();
        auto f = make_future(*p);
        add_listener([p, map](const attempt<T>& result) {
            try {
                consolidate_fmap(p, result, map);
            } catch(...) {
                p->set_failure(std::current_exception());
            }
        }, exec);
        return f; 
    }

    template <class R, class P>    
    void wait_completion(const std::chrono::duration<R,P>& timeout) const {
        _core->wait_completion(timeout);
    }

    template <class R, class P, class U = T>
    typename std::enable_if<!std::is_void<U>::value, const U&>::type
    wait_result(const std::chrono::duration<R,P>& timeout) const {
        wait_completion(timeout);
        return get();
    }

    template <class R, class P, class U = T>
    typename std::enable_if<std::is_void<U>::value>::type
    wait_result(const std::chrono::duration<R,P>& timeout) const {
        wait_completion(timeout);
        return get();
    }

private:

    struct core {
        std::list<listener> _listeners;
        option<attempt<T>> _result;
        mutable std::recursive_mutex _mutex;
        mutable std::condition_variable_any _completion_cond;

        bool is_completed() const { 
            std::lock_guard<std::recursive_mutex> lock(_mutex);
            return _result.is_defined(); 
        }

        template <class U = T>
        typename std::enable_if<!std::is_void<U>::value, const U&>::type
        get() const {
            std::lock_guard<std::recursive_mutex> lock(_mutex);
            if (is_completed()) {
                return _result.get().get();
            } else {
                throw uncompleted_future(
                    "cannot get value since future is still uncompleted");
            }
        }

        template <class U = T>
        typename std::enable_if<std::is_void<U>::value>::type
        get() const {
            std::lock_guard<std::recursive_mutex> lock(_mutex);
            if (is_completed()) {
                _result.get().get();
            } else {
                throw uncompleted_future(
                    "cannot get value since future is still uncompleted");
            }
        }

        void add_listener(const listener& l) {
            std::lock_guard<std::recursive_mutex> lock(_mutex);
            if (is_completed()) {
                l(_result.get());
            } else {
                _listeners.push_back(l);
            }
        }

        template <class R, class P>
        void wait_completion(const std::chrono::duration<R,P>& timeout) const {
            std::unique_lock<std::recursive_mutex> lock(_mutex);
            auto completion = std::bind(&core::is_completed, this);
            if (!_completion_cond.wait_for(lock, timeout, completion)) {
                throw future_timeout(
                    "future timeout while waiting for completion");
            }
        }

        void on_complete(const util::attempt<T>& result) {
            _result = make_some(result);
            for (auto l : _listeners) {
                try { l(result); }
                catch (...) {} // ignore exceptions thrown by listeners
            }

        }
    };

    using shared_core = std::shared_ptr<core>;

    shared_core _core;

    template <class U, class Func, class V = T>
    static typename std::enable_if<std::is_void<V>::value && std::is_void<U>::value>::type
    consolidate_map(const shared_promise<U>& p, const attempt<V>& result, Func map) {
        static_assert(std::is_same<decltype(map()), U>::value, 
            "cannot invoke map with a function that doesn't return T");
        result.get();
        map();
        p->set_success();
    }

    template <class U, class Func, class V = T>
    static typename std::enable_if<std::is_void<V>::value && !std::is_void<U>::value>::type
    consolidate_map(const shared_promise<U>& p, const attempt<V>& result, Func map) {
        static_assert(std::is_same<decltype(map()), U>::value, 
            "cannot invoke map with a function that doesn't return T");
        result.get();
        p->set_success(map());
    }

    template <class U, class Func, class V = T>
    static typename std::enable_if<!std::is_void<V>::value && std::is_void<U>::value>::type
    consolidate_map(const shared_promise<U>& p, const attempt<V>& result, Func map) {
        static_assert(std::is_same<decltype(map(result.get())), U>::value, 
            "cannot invoke map with a function that doesn't return T");
        map(result.get());
        p->set_success();
    }

    template <class U, class Func, class V = T>
    static typename std::enable_if<!std::is_void<V>::value && !std::is_void<U>::value>::type
    consolidate_map(const shared_promise<U>& p, const attempt<V>& result, Func map) {
        static_assert(std::is_same<decltype(map(result.get())), U>::value, 
            "cannot invoke map with a function that doesn't return T");
        p->set_success(map(result.get()));
    }

    template <class U, class Func, class V = T>
    static typename std::enable_if<!std::is_void<V>::value>::type
    consolidate_fmap(const shared_promise<U>& p, const attempt<V>& result, Func map) {
        static_assert(std::is_same<decltype(map(result.get())), future<U>>::value, 
            "cannot invoke fmap with a function that doesn't return future<T>");
        auto mapped = map(result.get());
        mapped.add_listener([p](const attempt<U>& other_result) {
            p->set(other_result);
        });
    }

    template <class U, class Func, class V = T>
    static typename std::enable_if<std::is_void<V>::value>::type
    consolidate_fmap(const shared_promise<U>& p, const attempt<V>& result, Func map) {
        static_assert(std::is_same<decltype(map()), future<U>>::value, 
            "cannot invoke fmap with a function that doesn't return future<T>");
        result.get();
        auto mapped = map();
        mapped.add_listener([p](const attempt<U>& other_result) {
            p->set(other_result);
        });
    }
};

template <class T>
future<T> make_future(promise<T>& p) {
    return future<T>(p);
}

template <class T>
typename std::enable_if<!std::is_void<T>::value, future<T>>::type
make_future_success(const T& t) {
    promise<T> p;
    auto f = make_future(p);
    p.set_success(t);
    return f;
}

template <class T>
typename std::enable_if<std::is_void<T>::value, future<T>>::type
make_future_success() {
    promise<T> p;
    auto f = make_future(p);
    p.set_success();
    return f;
}

template <class T, class Exception>
future<T> make_future_failure(const Exception& error) {
    promise<T> p;
    auto f = make_future(p);
    p.set_failure(error);
    return f;
}

}}

#endif

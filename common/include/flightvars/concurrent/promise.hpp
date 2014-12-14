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

#include <functional>
#include <list>

#include <flightvars/util/attempt.hpp>

namespace flightvars { namespace concurrent {

FV_DECL_EXCEPTION(bad_promise);
FV_DECL_EXCEPTION(broken_promise);

using namespace util;

template <class T>
class promise {
public:

    static_assert(std::is_void<T>::value || std::is_copy_constructible<T>::value, 
        "cannot instantiate a promise with a non-copy constructible type");

    using listener = std::function<void(const util::attempt<T>&)>;

    promise() : _valid(true) {}

    promise(const promise& other) = delete;

    promise(promise&& other) : 
            _valid(other._valid), _listeners(std::move(other._listeners)) {
        other._valid = false;
    }

    ~promise() {
        if (is_valid()) {
            set_failure(broken_promise(
                "promise deleted before setting either value or error"));
        }
    }

    bool is_valid() const { return _valid; }


    template <class U = T>
    typename std::enable_if<!std::is_void<U>::value>::type 
    set_success(const U& value) {
        check_valid();
        notify_success(value);
    }

    template <class U = T>
    typename std::enable_if<std::is_void<U>::value>::type 
    set_success() {
        check_valid();
        notify_success();
    }

    void set_failure(const std::exception_ptr& error) {
        check_valid();
        notify_failure(error);
    }

    template <class E>
    void set_failure(const E& error) {
        check_valid();
        notify_failure(std::make_exception_ptr(error));
    }

    template <class U = T>
    typename std::enable_if<!std::is_void<U>::value>::type 
    set(const attempt<T>& result) {
        check_valid();
        try {
            auto value = result.get();
            notify_success(value);
        } catch (...) {
            auto error = std::current_exception();
            notify_failure(error);
        }
    }

    template <class U = T>
    typename std::enable_if<std::is_void<U>::value>::type 
    set(const attempt<T>& result) {
        check_valid();
        try {
            result.get();
            notify_success();
        } catch (...) {
            auto error = std::current_exception();
            notify_failure(error);
        }
    }

    void add_listener(const listener& l) {
        check_valid();
        _listeners.push_back(l);
    }

private:

    bool _valid;
    std::list<listener> _listeners;    

    void check_valid() {
        if (!is_valid()) {
            throw bad_promise("cannot operate on invalid promise");
        }        
    }

    template <class U = T>
    typename std::enable_if<!std::is_void<U>::value>::type 
    notify_success(const U& value) {
        _valid = false;
        for (auto l : _listeners) {
            try { l(util::make_success(value)); }
            catch (...) {}
        }
    }

    template <class U = T>
    typename std::enable_if<std::is_void<U>::value>::type 
    notify_success() {
        _valid = false;
        for (auto l : _listeners) {
            try { l(util::make_success<T>()); }
            catch (...) {}
        }
    }

    void notify_failure(const std::exception_ptr& error) {
        _valid = false;
        for (auto l : _listeners) {
            try { l(util::make_failure<T>(error)); }
            catch (...) {}
        }
    }
};

template <class T>
using shared_promise = std::shared_ptr<promise<T>>;

template <class T>
shared_promise<T> make_shared_promise() {
    return std::make_shared<promise<T>>();
}

}}

#endif


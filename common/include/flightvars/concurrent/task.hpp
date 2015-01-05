/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_CONCURRENT_TASK_H
#define FLIGHTVARS_CONCURRENT_TASK_H

#include <flightvars/util/exception.hpp>

namespace flightvars { namespace concurrent {

/**
 * A functor object that wraps a function with its params.
 *
 * This class provides an abstraction of a task: a functor that collects a function with its
 * arguments with move semantics and invokes that function when its call operator is invoked.
 * It is a good replacement of std::bind with full move semantics.
 *
 * `task` class is CopyConstructible in order to avoid issues while using it with executors.
 */
template <class ...Args>
class task_wrapper;

template <class T1>
class task_wrapper<T1> {
public:

    template <class Func>
    task_wrapper(Func&& f, T1&& arg1) : _target(std::forward<Func>(f)),
                                        _arg1(std::make_shared<T1>(std::move(arg1))) {}

    task_wrapper(const task_wrapper&) = default;
    task_wrapper(task_wrapper&& other) = default;

    task_wrapper& operator = (const task_wrapper&) = default;
    task_wrapper& operator = (task_wrapper&&) = default;

    void operator()(void) {
        _target(std::move(*_arg1));
    }

private:

    std::function<void(T1)> _target;
    std::shared_ptr<T1> _arg1;
};

template <class Func, class ...Args>
task_wrapper<Args...> make_task(Func&& f, Args&&... args) {
    return task_wrapper<Args...>(std::forward<Func>(f), std::forward<Args>(args)...);
}

}}

#endif

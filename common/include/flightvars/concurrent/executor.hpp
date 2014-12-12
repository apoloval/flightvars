/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_CONCURRENT_EXECUTOR_H
#define FLIGHTVARS_CONCURRENT_EXECUTOR_H

#include <functional>

#include <boost/asio.hpp>

namespace flightvars { namespace concurrent {

class same_thread_executor {
public:

    same_thread_executor() = default;
    same_thread_executor(const same_thread_executor&) = default;
    same_thread_executor(same_thread_executor&&) = default;

    template <class Task>
    void execute(Task task) { task(); }
};

class asio_service_executor {
public:

    asio_service_executor() : _service(std::make_shared<boost::asio::io_service>()) {}
    asio_service_executor(const asio_service_executor&) = default;
    asio_service_executor(asio_service_executor&&) = default;

    boost::asio::io_service& io_service() { return *_service; }

    void run() { _service->run(); }

    template <class Task>
    void execute(Task task) { _service->post(task); }

private:

    std::shared_ptr<boost::asio::io_service> _service;
};

template <class Executor, class Func, class ...Args>
void run(Executor& exec, Func func, Args... args) {
    auto f = std::bind(func, args...);
    exec.execute(f);
}

}}

#endif

/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <boost/test/unit_test.hpp>

#include <flightvars/concurrent/executor.hpp>

using namespace flightvars::concurrent;

BOOST_AUTO_TEST_SUITE(ConcurrentSameThreadExecutor)

BOOST_AUTO_TEST_CASE(MustExecuteTask) {
    same_thread_executor exec;
    bool is_invoked = false;
    exec.execute([&is_invoked]() { is_invoked = true; });
    BOOST_CHECK(is_invoked);
}

BOOST_AUTO_TEST_SUITE_END()

BOOST_AUTO_TEST_SUITE(ConcurrentAsioServiceExecutor)

BOOST_AUTO_TEST_CASE(MustExecuteTask) {
    asio_service_executor exec;
    bool is_invoked = false;
    exec.execute([&is_invoked]() { is_invoked = true; });
    exec.run();
    BOOST_CHECK(is_invoked);
}

BOOST_AUTO_TEST_SUITE_END()

BOOST_AUTO_TEST_SUITE(ConcurrentExecutorFunctions)

BOOST_AUTO_TEST_CASE(MustRunFunctionWithArgumentsInSameThread) {
    same_thread_executor exec;
    int num = 0;
    auto f = [&num](int n) { num = n; };
    run(exec, f, 2);
    BOOST_CHECK_EQUAL(2, num);
}

BOOST_AUTO_TEST_CASE(MustRunFunctionWithArgumentsInAsioService) {
    asio_service_executor exec;
    int num = 0;
    auto f = [&num](int n) { num = n; };
    run(exec, f, 2);
    exec.run();
    BOOST_CHECK_EQUAL(2, num);
}

BOOST_AUTO_TEST_SUITE_END()

/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <boost/test/unit_test.hpp>

#include <flightvars/concurrent/task.hpp>
#include <flightvars/util/noncopyable.hpp>

using namespace flightvars;
using namespace flightvars::concurrent;

BOOST_AUTO_TEST_SUITE(UtilTask)

BOOST_AUTO_TEST_CASE(MustInvokeLambdaTarget) {
    std::string r;
    auto target = [&r](util::noncopyable<std::string>&& str) { r = *str; };
    auto str = util::make_noncopyable<std::string>("Hello!");
    auto t = make_task(target, std::move(str));
    t();

    BOOST_CHECK_EQUAL("Hello!", r);
    BOOST_CHECK_EQUAL("", *str);
}

BOOST_AUTO_TEST_CASE(MustMoveArgumentsBeforeInvocation) {
    std::string r;
    auto target = [&r](util::noncopyable<std::string>&& str) { r = *str; };
    auto str = util::make_noncopyable<std::string>("Hello!");
    auto t = make_task(target, std::move(str));

    BOOST_CHECK_EQUAL("", r);
    BOOST_CHECK_EQUAL("", *str);
}

BOOST_AUTO_TEST_SUITE_END()

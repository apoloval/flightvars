/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <boost/test/unit_test.hpp>

#include <flightvars/concurrent/promise.hpp>

using namespace flightvars;
using namespace flightvars::concurrent;

BOOST_AUTO_TEST_SUITE(ConcurrentPromise)

FV_DECL_EXCEPTION(custom_exception);

BOOST_AUTO_TEST_CASE(MustFailToObtainFutureTwice) {
    promise<std::string> p;
    auto f = p.get_future();
    BOOST_CHECK_THROW(p.get_future(), future_already_retrieved);
}

BOOST_AUTO_TEST_CASE(MustResetAfterSetValue) {
    promise<std::string> p;
    p.set_value("Hello!");
    BOOST_CHECK(!p.valid());
}

BOOST_AUTO_TEST_CASE(MustResetAfterSetException) {
    promise<std::string> p;
    p.set_failure(custom_exception("bad luck"));
    BOOST_CHECK(!p.valid());
}

BOOST_AUTO_TEST_SUITE_END()

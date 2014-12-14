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

using namespace flightvars::concurrent;

BOOST_AUTO_TEST_SUITE(ConcurrentPromise)

FV_DECL_EXCEPTION(custom_exception);

BOOST_AUTO_TEST_CASE(MustBeValidAfterDefaultConstruction) {
    promise<int> p;

    BOOST_CHECK(p.is_valid());
}

BOOST_AUTO_TEST_CASE(MustBeMoveable) {
    promise<int> p1;
    promise<int> p2(std::move(p1));

    BOOST_CHECK(!p1.is_valid());
    BOOST_CHECK_THROW(p1.set_success(10), bad_promise);
    BOOST_CHECK(p2.is_valid());
    BOOST_CHECK_NO_THROW(p2.set_success(10));
}

BOOST_AUTO_TEST_CASE(MustInvalidateAfterSetSuccess) {
    promise<int> p;

    p.set_success(10);
    BOOST_CHECK(!p.is_valid());
    BOOST_CHECK_THROW(p.set_success(11), bad_promise);
}

BOOST_AUTO_TEST_CASE(MustInvalidateAfterSetFailure) {
    promise<int> p;

    p.set_failure(custom_exception("something went wrong"));
    BOOST_CHECK(!p.is_valid());
    BOOST_CHECK_THROW(p.set_success(11), bad_promise);
}

BOOST_AUTO_TEST_CASE(MustInvokeListenersOnSetSuccess) {
    promise<int> p;
    auto result = make_none<attempt<int>>();
    p.add_listener([&result](const attempt<int>& r) {
        result = make_some(r);
    });
    BOOST_CHECK(!result.is_defined());

    p.set_success(10);
    BOOST_CHECK_EQUAL(10, result.get().get());

}

BOOST_AUTO_TEST_CASE(MustInvokeListenersOnSetFailure) {
    promise<int> p;
    auto result = make_none<attempt<int>>();
    p.add_listener([&result](const attempt<int>& r) {
        result = make_some(r);
    });
    BOOST_CHECK(!result.is_defined());

    p.set_failure(custom_exception("something went wrong"));
    BOOST_CHECK_THROW(result.get().get(), custom_exception);

}

BOOST_AUTO_TEST_SUITE_END()

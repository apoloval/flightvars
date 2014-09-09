/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <boost/test/unit_test.hpp>

#include <flightvars/util/attempt.hpp>

using namespace flightvars::util;

BOOST_AUTO_TEST_SUITE(UtilAttempt)

FV_DECL_EXCEPTION(custom_exception);

BOOST_AUTO_TEST_CASE(MustWrapAValueWhenInitFromSuccess)
{
    auto a = make_success(10);
    BOOST_CHECK(a.is_success());
    BOOST_CHECK(!a.is_failure());
    BOOST_CHECK_EQUAL(10, a.get());
}

BOOST_AUTO_TEST_CASE(MustWrapAnErrorWhenInitFromFailure)
{
    auto a = make_failure<int>(custom_exception("something went wrong"));
    BOOST_CHECK(!a.is_success());
    BOOST_CHECK(a.is_failure());
    BOOST_CHECK_THROW(a.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustBeCopyable) {
    auto a1 = make_success(10);
    auto a2 = make_failure<int>(custom_exception("something went wrong"));
    attempt<int> a3(a1);
    attempt<int> a4(a2);
    auto a5 = a1;
    auto a6 = a2;

    BOOST_CHECK_EQUAL(10, a3.get());
    BOOST_CHECK_THROW(a4.get(), custom_exception);
    BOOST_CHECK_EQUAL(10, a5.get());
    BOOST_CHECK_THROW(a6.get(), custom_exception);
}

BOOST_AUTO_TEST_SUITE_END()

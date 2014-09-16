/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <boost/test/unit_test.hpp>

#include <flightvars/util/format.hpp>

using namespace flightvars::util;

BOOST_AUTO_TEST_SUITE(UtilFormat)

BOOST_AUTO_TEST_CASE(MustFormatNoArgs) {
    BOOST_CHECK_EQUAL("Number", format("Number"));
}

BOOST_AUTO_TEST_CASE(MustFormatOneArg) {
    BOOST_CHECK_EQUAL("Number: 7", format("Number: %d", 7));
}

BOOST_AUTO_TEST_CASE(MustFormatManyArgs) {
    BOOST_CHECK_EQUAL("Numbers: 1, 2, 3", format("Numbers: %d, %d, %d", 1, 2, 3));
}

BOOST_AUTO_TEST_SUITE_END()

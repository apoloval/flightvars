/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <boost/test/unit_test.hpp>

#include <flightvars/util/either.hpp>

#include "noncopyable.hpp"

using namespace flightvars::util;

BOOST_AUTO_TEST_SUITE(UtilEither)

BOOST_AUTO_TEST_CASE(MustConstructWithNone) {
    auto val = either<int, bool>();
    BOOST_CHECK(!val.has_left());
    BOOST_CHECK(!val.has_right());
    BOOST_CHECK_THROW(val.left(), either_error);
    BOOST_CHECK_THROW(val.right(), either_error);
}

BOOST_AUTO_TEST_CASE(MustConstructWithLeftCopy) {
    auto val = either<int, bool>(7);
    BOOST_CHECK(val.has_left());
    BOOST_CHECK(!val.has_right());
    BOOST_CHECK_EQUAL(7, val.left());
    BOOST_CHECK_THROW(val.right(), either_error);
}

BOOST_AUTO_TEST_CASE(MustConstructWithLeftMove) {
    auto str = std::string("Hello!");
    auto val = either<noncopyable<std::string>, bool>(std::move(str));
    BOOST_CHECK(val.has_left());
    BOOST_CHECK(!val.has_right());
    BOOST_CHECK_EQUAL("", str);
    BOOST_CHECK_EQUAL("Hello!", *val.left());
    BOOST_CHECK_THROW(val.right(), either_error);
}

BOOST_AUTO_TEST_CASE(MustConstructWithRightCopy) {
    auto val = either<int, bool>(true);
    BOOST_CHECK(!val.has_left());
    BOOST_CHECK(val.has_right());
    BOOST_CHECK_EQUAL(true, val.right());
    BOOST_CHECK_THROW(val.left(), either_error);
}

BOOST_AUTO_TEST_CASE(MustConstructWithRightMove) {
    auto str = std::string("Hello!");
    auto val = either<int, noncopyable<std::string>>(std::move(str));
    BOOST_CHECK(!val.has_left());
    BOOST_CHECK_EQUAL("Hello!", *val.right());
    BOOST_CHECK_THROW(val.left(), either_error);
}

BOOST_AUTO_TEST_CASE(MustCopyConstruct) {
    auto val1 = either<int, bool>(7);
    auto val2 = val1;
    BOOST_CHECK_EQUAL(7, val2.left());
}

BOOST_AUTO_TEST_CASE(MustMoveConstruct) {
    auto val1 = either<noncopyable<int>, noncopyable<bool>>(make_noncopyable(7));
    auto val2 = std::move(val1);
    BOOST_CHECK_EQUAL(7, *val2.left());
    BOOST_CHECK(!val1.has_left());
    BOOST_CHECK(!val1.has_right());
}

BOOST_AUTO_TEST_CASE(MustCopyAssign) {
    auto val1 = either<int, bool>(7);
    auto val2 = either<int, bool>();
    val2 = val1;
    BOOST_CHECK_EQUAL(7, val2.left());
}

BOOST_AUTO_TEST_CASE(MustMoveAssign) {
    auto val1 = either<noncopyable<int>, noncopyable<bool>>(make_noncopyable(7));
    auto val2 = either<noncopyable<int>, noncopyable<bool>>();
    val2 = std::move(val1);
    BOOST_CHECK_EQUAL(7, *val2.left());
    BOOST_CHECK(!val1.has_left());
    BOOST_CHECK(!val1.has_right());
}

BOOST_AUTO_TEST_CASE(MustResetBoth) {
    auto val = either<int, bool>(true);
    val.reset();
    BOOST_CHECK(!val.has_left());
    BOOST_CHECK(!val.has_right());
}

BOOST_AUTO_TEST_CASE(MustResetLeftByCopy) {
    auto val = either<int, bool>(true);
    val.reset(7);
    BOOST_CHECK_EQUAL(7, val.left());
    BOOST_CHECK(!val.has_right());
}

BOOST_AUTO_TEST_CASE(MustResetLeftByMove) {
    auto val = either<noncopyable<std::string>, bool>(true);
    auto str = std::string("Hello!");
    val.reset(make_noncopyable(std::move(str)));
    BOOST_CHECK_EQUAL("Hello!", *val.left());
    BOOST_CHECK_EQUAL("", str);
    BOOST_CHECK(!val.has_right());
}

BOOST_AUTO_TEST_CASE(MustResetRightByCopy) {
    auto val = either<int, bool>(7);
    val.reset(true);
    BOOST_CHECK_EQUAL(true, val.right());
    BOOST_CHECK(!val.has_left());
}

BOOST_AUTO_TEST_CASE(MustResetRightByMove) {
    auto val = either<int, noncopyable<std::string>>(7);
    auto str = std::string("Hello!");
    val.reset(make_noncopyable(std::move(str)));
    BOOST_CHECK_EQUAL("Hello!", *val.right());
    BOOST_CHECK_EQUAL("", str);
    BOOST_CHECK(!val.has_left());
}

BOOST_AUTO_TEST_CASE(MustExtractLeft) {
    auto val = either<int, std::string>(7);
    auto i = val.extract_left();
    BOOST_CHECK_EQUAL(7, i);
    BOOST_CHECK(!val.valid());
}

BOOST_AUTO_TEST_CASE(MustExtractRight) {
    auto val = either<int, std::string>("Hello!");
    auto str = val.extract_right();
    BOOST_CHECK_EQUAL("Hello!", str);
    BOOST_CHECK(!val.valid());
}

BOOST_AUTO_TEST_SUITE_END()

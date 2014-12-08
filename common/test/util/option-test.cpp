/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <boost/test/unit_test.hpp>

#include <flightvars/util/option.hpp>

using namespace flightvars::util;

BOOST_AUTO_TEST_SUITE(UtilOption)

struct parent {
    int data;
    parent(int d) : data(d) {}
};

struct child : public parent {
    child(int d) : parent(d) {}
};

BOOST_AUTO_TEST_CASE(MustBeDefinedWhenInitFromValue)
{
    option<int> opt(10);
    BOOST_CHECK(opt.is_defined());
}

BOOST_AUTO_TEST_CASE(MustNotBeDefinedWhenInitFromNothing)
{
    option<int> opt;
    BOOST_CHECK(!opt.is_defined());
}

BOOST_AUTO_TEST_CASE(MustGetWhenDefined)
{
    option<int> opt(10);
    BOOST_CHECK_EQUAL(10, opt.get());
}

BOOST_AUTO_TEST_CASE(MustThrowOnGetWhenUndefined)
{
    option<int> opt;
    BOOST_CHECK_THROW(opt.get(), option_undefined);
}

BOOST_AUTO_TEST_CASE(MustGetByMovement)
{
    option<std::string> opt("Hello!");
    auto s = std::move(opt.get());
    BOOST_CHECK_EQUAL("Hello!", s);
    BOOST_CHECK_EQUAL("", opt.get());
}

BOOST_AUTO_TEST_CASE(MustExtract)
{
    option<std::string> opt("Hello!");
    auto s = std::move(opt.extract());
    BOOST_CHECK_EQUAL("Hello!", s);
    BOOST_CHECK(!opt.is_defined());
}

BOOST_AUTO_TEST_CASE(MustDefineWithSet)
{
    option<int> opt;
    opt.set(10);
    BOOST_CHECK(opt.is_defined());
    BOOST_CHECK_EQUAL(10, opt.get());
}

BOOST_AUTO_TEST_CASE(MustHonourCopy)
{
    option<int> opt1(10);
    option<int> opt2(opt1);

    BOOST_CHECK(opt1.is_defined());
    BOOST_CHECK(opt2.is_defined());
    BOOST_CHECK_EQUAL(10, opt1.get());
    BOOST_CHECK_EQUAL(10, opt2.get());
}

BOOST_AUTO_TEST_CASE(MustHonourMove)
{
    option<int> opt1(10);
    option<int> opt2(std::move(opt1));

    BOOST_CHECK(!opt1.is_defined());
    BOOST_CHECK(opt2.is_defined());
    BOOST_CHECK_EQUAL(10, opt2.get());
}

BOOST_AUTO_TEST_CASE(MustHonourCovariantCopy)
{
    option<child> opt1(child(10));
    option<parent> opt2(opt1);

    BOOST_CHECK(opt1.is_defined());
    BOOST_CHECK(opt2.is_defined());
    BOOST_CHECK_EQUAL(10, opt1.get().data);
    BOOST_CHECK_EQUAL(10, opt2.get().data);
}

BOOST_AUTO_TEST_CASE(MustHonourCovariantMove)
{
    option<child> opt1(child(10));
    option<parent> opt2(std::move(opt1));

    BOOST_CHECK(!opt1.is_defined());
    BOOST_CHECK(opt2.is_defined());
    BOOST_CHECK_EQUAL(10, opt2.get().data);
}

BOOST_AUTO_TEST_SUITE_END()

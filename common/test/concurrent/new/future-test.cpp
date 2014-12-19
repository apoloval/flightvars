/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <boost/test/unit_test.hpp>

#include <flightvars/concurrent/new/future.hpp>
#include <flightvars/concurrent/new/promise.hpp>

using namespace flightvars;
using namespace flightvars::concurrent::newwave;

BOOST_AUTO_TEST_SUITE(ConcurrentNewFuture)

FV_DECL_EXCEPTION(custom_exception);

BOOST_AUTO_TEST_CASE(MustInitInvalidWithDefaultConstructor) {
    future<std::string> f;
    BOOST_CHECK(!f.valid());
}

BOOST_AUTO_TEST_CASE(MustThrowOnGetWhenNotValid) {
    future<std::string> f;
    BOOST_CHECK_THROW(f.get(), bad_future);
}

BOOST_AUTO_TEST_CASE(MustThrowOnWaitWhenNotValid) {
    future<std::string> f;
    BOOST_CHECK_THROW(f.wait(), bad_future);
}

BOOST_AUTO_TEST_CASE(MustThrowOnWaitForWhenNotValid) {
    future<std::string> f;
    BOOST_CHECK_THROW(f.wait_for(std::chrono::seconds(1)), bad_future);
}

BOOST_AUTO_TEST_CASE(MustBeIncompleteBeforePromiseIsSet) {
    promise<std::string> p;
    auto f = p.get_future();
    BOOST_CHECK(!f.is_completed());
}

BOOST_AUTO_TEST_CASE(MustBeCompletedAfterPromiseIsSet) {
    promise<std::string> p;
    auto f = p.get_future();
    p.set_value("Hello!");
    BOOST_CHECK(f.is_completed());
}

BOOST_AUTO_TEST_CASE(MustGetWhenPromiseIsSet) {
    promise<std::string> p;
    auto f = p.get_future();
    p.set_value("Hello!");
    BOOST_CHECK_EQUAL("Hello!", f.get());
}

BOOST_AUTO_TEST_CASE(MustWaitForWhenPromiseIsSet) {
    promise<std::string> p;
    auto f = p.get_future();
    p.set_value("Hello!");
    BOOST_CHECK_NO_THROW(f.wait_for(std::chrono::seconds(1)));
}

BOOST_AUTO_TEST_CASE(MustThrowWaitForWhenPromiseIsNotSet) {
    promise<std::string> p;
    auto f = p.get_future();
    BOOST_CHECK_THROW(f.wait_for(std::chrono::milliseconds(25)), future_timeout);
}

BOOST_AUTO_TEST_CASE(MustInvalidateSourceAfterMoveConstruct) {
    promise<std::string> p;
    auto f1 = p.get_future();
    auto f2 = std::move(f1);
    BOOST_CHECK(!f1.valid());
    BOOST_CHECK(f2.valid());
}

BOOST_AUTO_TEST_CASE(MustInvalidateSourceAfterMoveAssign) {
    promise<std::string> p;
    auto f1 = p.get_future();
    future<std::string> f2;
    f2 = std::move(f1);
    BOOST_CHECK(!f1.valid());
    BOOST_CHECK(f2.valid());
}

BOOST_AUTO_TEST_CASE(MustOperateNormallyAfterMoveConstruct) {
    promise<std::string> p;
    auto f1 = p.get_future();
    auto f2 = std::move(f1);
    p.set_value("Hello!");
    BOOST_CHECK_EQUAL("Hello!", f2.get());
}

BOOST_AUTO_TEST_CASE(MustOperateNormallyAfterMoveAssign) {
    promise<std::string> p;
    auto f1 = p.get_future();
    future<std::string> f2;
    f2 = std::move(f1);
    p.set_value("Hello!");
    BOOST_CHECK_EQUAL("Hello!", f2.get());
}

BOOST_AUTO_TEST_CASE(MustSetValueFromVoidPromise) {
    promise<void> p;
    auto f = p.get_future();
    p.set_value();
    BOOST_CHECK_NO_THROW(f.get());
}

BOOST_AUTO_TEST_CASE(MustSetExceptionFromVoidPromise) {
    promise<void> p;
    auto f = p.get_future();
    p.set_failure(custom_exception("failure"));
    BOOST_CHECK_THROW(f.get(), custom_exception);
}

BOOST_AUTO_TEST_SUITE_END()

/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <boost/test/unit_test.hpp>

#include <flightvars/concurrent/future.hpp>
#include <flightvars/concurrent/promise.hpp>
#include <flightvars/util/noncopyable.hpp>

using namespace flightvars;
using namespace flightvars::concurrent;

BOOST_AUTO_TEST_SUITE(ConcurrentFuture)

FV_DECL_EXCEPTION(custom_exception);

BOOST_AUTO_TEST_CASE(MustMakeFutureSuccess) {
    auto f = make_future_success(util::make_noncopyable<std::string>("Hello!"));
    BOOST_CHECK_EQUAL("Hello!", *f.get());
}

BOOST_AUTO_TEST_CASE(MustMakeFutureSuccessVoid) {
    auto f = make_future_success<void>();
    BOOST_CHECK_NO_THROW(f.get());
}

BOOST_AUTO_TEST_CASE(MustMakeFutureFailure) {
    auto f = make_future_failure<util::noncopyable<std::string>>(custom_exception("failed"));
    BOOST_CHECK_THROW(f.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustInitInvalidWithDefaultConstructor) {
    future<util::noncopyable<std::string>> f;
    BOOST_CHECK(!f.valid());
}

BOOST_AUTO_TEST_CASE(MustThrowOnGetWhenNotValid) {
    future<util::noncopyable<std::string>> f;
    BOOST_CHECK_THROW(f.get(), bad_future);
}

BOOST_AUTO_TEST_CASE(MustThrowOnWaitWhenNotValid) {
    future<util::noncopyable<std::string>> f;
    BOOST_CHECK_THROW(f.wait(), bad_future);
}

BOOST_AUTO_TEST_CASE(MustThrowOnWaitForWhenNotValid) {
    future<util::noncopyable<std::string>> f;
    BOOST_CHECK_THROW(f.wait_for(std::chrono::seconds(1)), bad_future);
}

BOOST_AUTO_TEST_CASE(MustBeIncompleteBeforePromiseIsSet) {
    promise<util::noncopyable<std::string>> p;
    auto f = p.get_future();
    BOOST_CHECK(!f.is_completed());
}

BOOST_AUTO_TEST_CASE(MustBeCompletedAfterPromiseIsSet) {
    promise<util::noncopyable<std::string>> p;
    auto f = p.get_future();
    p.set_value("Hello!");
    BOOST_CHECK(f.is_completed());
}

BOOST_AUTO_TEST_CASE(MustGetWhenPromiseIsSet) {
    promise<util::noncopyable<std::string>> p;
    auto f = p.get_future();
    p.set_value("Hello!");
    BOOST_CHECK_EQUAL("Hello!", *f.get());
}

BOOST_AUTO_TEST_CASE(MustWaitForWhenPromiseIsSet) {
    promise<util::noncopyable<std::string>> p;
    auto f = p.get_future();
    p.set_value("Hello!");
    BOOST_CHECK_NO_THROW(f.wait_for(std::chrono::seconds(1)));
}

BOOST_AUTO_TEST_CASE(MustThrowWaitForWhenPromiseIsNotSet) {
    promise<util::noncopyable<std::string>> p;
    auto f = p.get_future();
    BOOST_CHECK_THROW(f.wait_for(std::chrono::milliseconds(25)), future_timeout);
}

BOOST_AUTO_TEST_CASE(MustInvalidateSourceAfterMoveConstruct) {
    promise<util::noncopyable<std::string>> p;
    auto f1 = p.get_future();
    auto f2 = std::move(f1);
    BOOST_CHECK(!f1.valid());
    BOOST_CHECK(f2.valid());
}

BOOST_AUTO_TEST_CASE(MustInvalidateSourceAfterMoveAssign) {
    promise<util::noncopyable<std::string>> p;
    auto f1 = p.get_future();
    future<util::noncopyable<std::string>> f2;
    f2 = std::move(f1);
    BOOST_CHECK(!f1.valid());
    BOOST_CHECK(f2.valid());
}

BOOST_AUTO_TEST_CASE(MustOperateNormallyAfterMoveConstruct) {
    promise<util::noncopyable<std::string>> p;
    auto f1 = p.get_future();
    auto f2 = std::move(f1);
    p.set_value("Hello!");
    BOOST_CHECK_EQUAL("Hello!", *f2.get());
}

BOOST_AUTO_TEST_CASE(MustOperateNormallyAfterMoveAssign) {
    promise<util::noncopyable<std::string>> p;
    auto f1 = p.get_future();
    future<util::noncopyable<std::string>> f2;
    f2 = std::move(f1);
    p.set_value("Hello!");
    BOOST_CHECK_EQUAL("Hello!", *f2.get());
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

BOOST_AUTO_TEST_CASE(MustBeInvalidAfterThen) {
    promise<util::noncopyable<std::string>> p;
    auto f1 = p.get_future();
    auto f2 = f1.then([](const util::noncopyable<std::string>& s) { return s.get().length(); });
    BOOST_CHECK(!f1.valid());
    BOOST_CHECK(f2.valid());
}

BOOST_AUTO_TEST_CASE(MustBeInvalidAfterNext) {
    promise<util::noncopyable<std::string>> p;
    auto f1 = p.get_future();
    auto f2 = f1.next<std::size_t>([](const util::noncopyable<std::string>& s) {
        return make_future_success(s.get().length());
    });
    BOOST_CHECK(!f1.valid());
    BOOST_CHECK(f2.valid());
}

BOOST_AUTO_TEST_CASE(MustBeInvalidAfterFinally) {
    promise<util::noncopyable<std::string>> p;
    auto f = p.get_future();
    util::attempt<util::noncopyable<std::string>> result;
    f.finally([&](util::attempt<util::noncopyable<std::string>> r) {
        result = std::move(r);
    });
    BOOST_CHECK(!f.valid());
}

BOOST_AUTO_TEST_CASE(MustGetValueOnThen) {
    promise<util::noncopyable<std::string>> p;
    auto f1 = p.get_future();
    auto f2 = f1.then([](const util::noncopyable<std::string>& s) { return s.get().length(); });
    p.set_value("Hello!");
    BOOST_CHECK_EQUAL(6, f2.get());
}

BOOST_AUTO_TEST_CASE(MustGetValueOnThenAfterResult) {
    auto f1 = make_future_success(util::make_noncopyable<std::string>("Hello!"));
    auto f2 = f1.then([](const util::noncopyable<std::string>& s) { return s.get().length(); });
    BOOST_CHECK_EQUAL(6, f2.get());
}

BOOST_AUTO_TEST_CASE(MustGetValueOnThenVoid) {
    promise<util::noncopyable<std::string>> p;
    auto f1 = p.get_future();
    auto f2 = f1.then([](const util::noncopyable<std::string>& s) {});
    p.set_value("Hello!");
    BOOST_CHECK_NO_THROW(f2.get());
}

BOOST_AUTO_TEST_CASE(MustGetValueOnNext) {
    promise<util::noncopyable<std::string>> p;
    auto f1 = p.get_future();
    auto f2 = f1.next<std::size_t>([](const util::noncopyable<std::string>& s) {
        return make_future_success(s.get().length());
    });
    p.set_value("Hello!");
    BOOST_CHECK_EQUAL(6, f2.get());
}

BOOST_AUTO_TEST_CASE(MustGetValueOnNextAfterResult) {
    auto f1 = make_future_success(util::make_noncopyable<std::string>("Hello!"));
    auto f2 = f1.next<std::size_t>([](const util::noncopyable<std::string>& s) {
        return make_future_success(s.get().length());
    });
    BOOST_CHECK_EQUAL(6, f2.get());
}

BOOST_AUTO_TEST_CASE(MustGetValueOnNextVoid) {
    promise<std::string> p;
    auto f1 = p.get_future();
    auto f2 = f1.next<void>([](std::string s) { return make_future_success<void>(); });
    p.set_value("Hello!");
    BOOST_CHECK_NO_THROW(f2.get());
}

BOOST_AUTO_TEST_CASE(MustThrowFailureOnNext) {
    promise<util::noncopyable<std::string>> p;
    auto f1 = p.get_future();
    auto f2 = f1.next<std::size_t>([](const util::noncopyable<std::string>& s) -> future<std::size_t> {
        throw custom_exception("failed");
    });
    p.set_value("Hello!");
    BOOST_CHECK_THROW(f2.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustThrowFailureOnNextVoid) {
    promise<util::noncopyable<std::string>> p;
    auto f1 = p.get_future();
    auto f2 = f1.next<void>([](const util::noncopyable<std::string>& s) -> future<void> {
        throw custom_exception("failed");
    });
    p.set_value("Hello!");
    BOOST_CHECK_THROW(f2.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustGetValueOnFinally) {
    promise<util::noncopyable<std::string>> p;
    auto f = p.get_future();
    util::attempt<util::noncopyable<std::string>> result;
    f.finally([&](util::attempt<util::noncopyable<std::string>> r) {
        result = std::move(r);
    });
    p.set_value("Hello!");
    BOOST_CHECK_EQUAL("Hello!", *result.get());
}

BOOST_AUTO_TEST_CASE(MustGetValueOnFinallyAfterResult) {
    promise<util::noncopyable<std::string>> p;
    auto f = p.get_future();
    p.set_value("Hello!");
    util::attempt<util::noncopyable<std::string>> result;
    f.finally([&](util::attempt<util::noncopyable<std::string>> r) {
        result = std::move(r);
    });
    BOOST_CHECK_EQUAL("Hello!", *result.get());
}

BOOST_AUTO_TEST_SUITE_END()

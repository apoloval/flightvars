/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <boost/test/unit_test.hpp>

#include <thread>

#include <flightvars/concurrent/future.hpp>
#include <flightvars/concurrent/promise.hpp>

using namespace flightvars::concurrent;

BOOST_AUTO_TEST_SUITE(ConcurrentFuture)

FV_DECL_EXCEPTION(custom_exception);

BOOST_AUTO_TEST_CASE(MustFailToConstructOnInvalidPromise)
{
    promise<int> p;
    p.set_success(10);
    BOOST_CHECK_THROW(future<int> f(p), bad_promise);
}

BOOST_AUTO_TEST_CASE(MustBeNotCompletedIfPromiseIsUnset)
{
    promise<int> p;
    future<int> f(p);
    BOOST_CHECK(!f.is_completed());
    BOOST_CHECK_THROW(f.get(), uncompleted_future);
}

BOOST_AUTO_TEST_CASE(MustBeCompletedIfPromiseIsSetToSuccess)
{
    promise<int> p;
    future<int> f(p);

    p.set_success(10);
    BOOST_CHECK(f.is_completed());
    BOOST_CHECK_EQUAL(10, f.get());
}

BOOST_AUTO_TEST_CASE(MustBeCompletedIfPromiseIsSetToFailure)
{
    promise<int> p;
    future<int> f(p);

    p.set_failure(custom_exception("something went wrong"));
    BOOST_CHECK(f.is_completed());
    BOOST_CHECK_THROW(f.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustBeCopyable)
{
    promise<int> p;
    future<int> f1(p);
    future<int> f2(f1);

    BOOST_CHECK(!f1.is_completed());
    BOOST_CHECK(!f2.is_completed());
    BOOST_CHECK_THROW(f1.get(), uncompleted_future);
    BOOST_CHECK_THROW(f2.get(), uncompleted_future);

    p.set_success(10);

    BOOST_CHECK(f1.is_completed());
    BOOST_CHECK(f2.is_completed());
    BOOST_CHECK_EQUAL(10, f1.get());
    BOOST_CHECK_EQUAL(10, f2.get());
}

BOOST_AUTO_TEST_CASE(MustCompleteWithBrokenPromiseIfPromiseIsLost)
{
    std::unique_ptr<future<int>> f;
    {
        promise<int> p;
        f.reset(new future<int>(p));
    }
    BOOST_CHECK(f->is_completed());
    BOOST_CHECK_THROW(f->get(), broken_promise);
}

BOOST_AUTO_TEST_CASE(MustIgnoreFutureIfLost)
{
    promise<int> p;
    {
        future<int> f(p);
    }
    BOOST_CHECK_NO_THROW(p.set_success(10));
}

BOOST_AUTO_TEST_CASE(MustInvokeListenersOnCompletion)
{
    promise<int> p;
    future<int> f(p);
    auto result = make_none<attempt<int>>();

    f.add_listener([&result](const attempt<int>& r) { result = r; });

    p.set_success(10);
    BOOST_CHECK_EQUAL(10, result.get().get());
}

BOOST_AUTO_TEST_CASE(MustInvokeNewListenersAfterCompletion)
{
    promise<int> p;
    future<int> f(p);
    p.set_success(10);
    auto result = make_none<attempt<int>>();

    f.add_listener([&result](const attempt<int>& r) { result = r; });

    BOOST_CHECK_EQUAL(10, result.get().get());
}

BOOST_AUTO_TEST_CASE(MustPropagateSuccessWithMap)
{
    promise<int> p;
    future<int> f1(p);
    auto f2 = f1.map<int>([](const int& value) { return value * 2; });

    p.set_success(10);
    BOOST_CHECK_EQUAL(20, f2.get());
}

BOOST_AUTO_TEST_CASE(MustPropagateFailureWithMap)
{
    promise<int> p;
    future<int> f1(p);
    auto f2 = f1.map<int>([](const int& value) { return value * 2; });

    p.set_failure(custom_exception("something went wrong"));
    BOOST_CHECK_THROW(f2.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustPropagateSuccessWithMapOnCompletedFuture)
{
    promise<int> p;
    future<int> f1(p);

    p.set_success(10);
    auto f2 = f1.map<int>([](const int& value) { return value * 2; });

    BOOST_CHECK_EQUAL(20, f2.get());
}

BOOST_AUTO_TEST_CASE(MustPropagateFailureWithMapOnCompletedFuture)
{
    promise<int> p;
    future<int> f1(p);
    p.set_failure(custom_exception("something went wrong"));
    auto f2 = f1.map<int>([](const int& value) { return value * 2; });

    BOOST_CHECK_THROW(f2.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustPropagateSuccessWithFlatMap)
{
    promise<int> p;
    future<int> f1(p);
    auto f2 = f1.fmap<float>([](const int& value) { 
        promise<float> p2;
        future<float> f2(p2);
        p2.set_success(value * 0.5f);
        return f2;
    });

    p.set_success(5);
    BOOST_CHECK_EQUAL(2.5f, f2.get());
}

BOOST_AUTO_TEST_CASE(MustPropagateFailureWithFlatMap)
{
    promise<int> p;
    future<int> f1(p);
    auto f2 = f1.fmap<int>([](const int& value) { 
        promise<int> p2;
        future<int> f2(p2);
        p2.set_success(value * 2);
        return f2;
    });

    p.set_failure(custom_exception("guess what? failure!"));
    BOOST_CHECK_THROW(f2.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustPropagateSuccessWithFlatMapOnCompletedFuture)
{
    promise<int> p;
    future<int> f1(p);
    p.set_success(10);
    auto f2 = f1.fmap<int>([](const int& value) { 
        promise<int> p2;
        future<int> f2(p2);
        p2.set_success(value * 2);
        return f2;
    });

    BOOST_CHECK_EQUAL(20, f2.get());
}

BOOST_AUTO_TEST_CASE(MustPropagateFailureWithFlatMapOnCompletedFuture)
{
    promise<int> p;
    future<int> f1(p);
    p.set_failure(custom_exception("guess what? failure!"));
    auto f2 = f1.fmap<int>([](const int& value) { 
        promise<int> p2;
        future<int> f2(p2);
        p2.set_success(value * 2);
        return f2;
    });

    BOOST_CHECK_THROW(f2.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustWaitForCompletion) {
    promise<int> p;
    future<int> f1(p);

    std::thread t([&p]() {
        std::this_thread::sleep_for(std::chrono::milliseconds(25));
        p.set_success(10);
    });

    f1.wait_completion(std::chrono::milliseconds(50));
    BOOST_CHECK_EQUAL(10, f1.get());

    t.join();
}

BOOST_AUTO_TEST_CASE(MustHonourWaitForCompletionTimeout) {
    promise<int> p;
    future<int> f1(p);

    std::thread t([&p]() {
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
        p.set_success(10);
    });

    BOOST_CHECK_THROW(
        f1.wait_completion(std::chrono::milliseconds(50)), 
        future_timeout);

    t.join();
}

BOOST_AUTO_TEST_CASE(MustWaitForResult) {
    promise<int> p;
    future<int> f1(p);

    std::thread t([&p]() {
        std::this_thread::sleep_for(std::chrono::milliseconds(25));
        p.set_success(10);
    });

    BOOST_CHECK_EQUAL(10, f1.wait_result(std::chrono::milliseconds(50)));
    t.join();
}

BOOST_AUTO_TEST_CASE(MustHonourWaitForResultTimeout) {
    promise<int> p;
    future<int> f1(p);

    std::thread t([&p]() {
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
        p.set_success(10);
    });

    BOOST_CHECK_THROW(
        f1.wait_result(std::chrono::milliseconds(50)),
        future_timeout);
    t.join();
}

BOOST_AUTO_TEST_SUITE_END()

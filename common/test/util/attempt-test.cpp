/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <boost/test/unit_test.hpp>

#include <flightvars/util/attempt.hpp>

#include "noncopyable.hpp"

using namespace flightvars::util;

BOOST_AUTO_TEST_SUITE(UtilAttempt)

FV_DECL_EXCEPTION(custom_exception);

BOOST_AUTO_TEST_CASE(MustWrapAValueWhenInitFromSuccess) {
    auto a = make_success(make_noncopyable(10));
    BOOST_CHECK(a.is_success());
    BOOST_CHECK(!a.is_failure());
    BOOST_CHECK_EQUAL(10, *a.get());
}

BOOST_AUTO_TEST_CASE(MustWrapAValueWhenInitFromVoidSuccess) {
    auto a = make_success<void>();
    BOOST_CHECK(a.is_success());
    BOOST_CHECK(!a.is_failure());
    BOOST_CHECK_NO_THROW(a.get());
}

BOOST_AUTO_TEST_CASE(MustWrapAnErrorWhenInitFromFailure) {
    auto a = make_failure<int>(custom_exception("something went wrong"));
    BOOST_CHECK(!a.is_success());
    BOOST_CHECK(a.is_failure());
    BOOST_CHECK_THROW(a.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustWrapAnErrorWhenInitFromVoidFailure)
{
    auto a = make_failure<void>(custom_exception("something went wrong"));
    BOOST_CHECK(!a.is_success());
    BOOST_CHECK(a.is_failure());
    BOOST_CHECK_THROW(a.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustBeCopyConstructible) {
    auto a1 = make_success(10);
    auto a2 = make_failure<int>(custom_exception("something went wrong"));
    attempt<int> a3(a1);
    attempt<int> a4(a2);

    BOOST_CHECK_EQUAL(10, a3.get());
    BOOST_CHECK_THROW(a4.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustBeCopyAssignable) {
    auto a1 = make_success(10);
    auto a2 = make_failure<int>(custom_exception("something went wrong"));
    attempt<int> a5;
    attempt<int> a6;
    a5 = a1;
    a6 = a2;

    BOOST_CHECK_EQUAL(10, a5.get());
    BOOST_CHECK_THROW(a6.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustBeMoveConstructible) {
    auto a1 = make_success(make_noncopyable(10));
    auto a2 = make_failure<noncopyable<int>>(custom_exception("something went wrong"));
    attempt<noncopyable<int>> a3(std::move(a1));
    attempt<noncopyable<int>> a4(std::move(a2));

    BOOST_CHECK(!a1.valid());
    BOOST_CHECK(!a2.valid());
    BOOST_CHECK_EQUAL(10, *a3.get());
    BOOST_CHECK_THROW(a4.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustBeMoveAssignable) {
    auto a1 = make_success(make_noncopyable(10));
    auto a2 = make_failure<noncopyable<int>>(custom_exception("something went wrong"));
    attempt<noncopyable<int>> a3, a4;
    a3 = std::move(a1);
    a4 = std::move(a2);

    BOOST_CHECK(!a1.valid());
    BOOST_CHECK(!a2.valid());
    BOOST_CHECK_EQUAL(10, *a3.get());
    BOOST_CHECK_THROW(a4.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustExtract) {
    auto a = make_success(make_noncopyable(10));
    auto val = a.extract();

    BOOST_CHECK_EQUAL(10, *val);
    BOOST_CHECK(!a.valid());
}

BOOST_AUTO_TEST_CASE(MustMapSuccess) {
    auto a = make_success(make_noncopyable<std::string>("Hello!"));
    auto b = a.map([](const noncopyable<std::string>& s) { return s.get().size(); });

    BOOST_CHECK(a.valid());
    BOOST_CHECK_EQUAL(6, b.get());
}

BOOST_AUTO_TEST_CASE(MustMapSuccessToVoid) {
    auto a = make_success(make_noncopyable<std::string>("Hello!"));
    auto b = a.map([](const noncopyable<std::string>& s) { });

    BOOST_CHECK(a.valid());
    BOOST_CHECK_NO_THROW(b.get());
}

BOOST_AUTO_TEST_CASE(MustMapFailure) {
    auto a = make_failure<noncopyable<std::string>>(custom_exception("something went wrong"));
    auto b = a.map([](const noncopyable<std::string>& s) { return s.get().size(); });

    BOOST_CHECK(a.valid());
    BOOST_CHECK_THROW(b.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustMapFailureToVoid) {
    auto a = make_failure<noncopyable<std::string>>(custom_exception("something went wrong"));
    auto b = a.map([](const noncopyable<std::string>& s) { });

    BOOST_CHECK(a.valid());
    BOOST_CHECK_THROW(b.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustMapVoidSuccess) {
    auto a = make_success<void>();
    auto b = a.map([]() { return 6; });

    BOOST_CHECK(a.valid());
    BOOST_CHECK_EQUAL(6, b.get());
}

BOOST_AUTO_TEST_CASE(MustMapVoidSuccessToVoid) {
    auto a = make_success<void>();
    auto b = a.map([]() {});

    BOOST_CHECK(a.valid());
    BOOST_CHECK_NO_THROW(b.get());
}

BOOST_AUTO_TEST_CASE(MustMapVoidFailure) {
    auto a = make_failure<void>(custom_exception("something went wrong"));
    auto b = a.map([]() { return 6; });

    BOOST_CHECK(a.valid());
    BOOST_CHECK_THROW(b.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustMapVoidFailureToVoid) {
    auto a = make_failure<void>(custom_exception("something went wrong"));
    auto b = a.map([]() {});

    BOOST_CHECK(a.valid());
    BOOST_CHECK_THROW(b.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustFMapSuccess) {
    auto a = make_success(make_noncopyable<std::string>("Hello!"));
    auto b = a.fmap<int>([](const noncopyable<std::string>& s) {
        return make_success<int>(s.get().size());
    });

    BOOST_CHECK(a.valid());
    BOOST_CHECK_EQUAL(6, b.get());
}

BOOST_AUTO_TEST_CASE(MustFMapSuccessToVoid) {
    auto a = make_success(make_noncopyable<std::string>("Hello!"));
    auto b = a.fmap<void>([](const noncopyable<std::string>& s) {
        return make_success<void>();
    });

    BOOST_CHECK(a.valid());
    BOOST_CHECK_NO_THROW(b.get());
}

BOOST_AUTO_TEST_CASE(MustFMapFailure) {
    auto a = make_failure<noncopyable<std::string>>(custom_exception("something went wrong"));
    auto b = a.fmap<int>([](const noncopyable<std::string>& s) {
        return make_success<int>(s.get().size());
    });

    BOOST_CHECK(a.valid());
    BOOST_CHECK_THROW(b.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustFMapFailureToVoid) {
    auto a = make_failure<noncopyable<std::string>>(custom_exception("something went wrong"));
    auto b = a.fmap<void>([](const noncopyable<std::string>& s) {
        return make_success<void>();
    });

    BOOST_CHECK(a.valid());
    BOOST_CHECK_THROW(b.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustFMapVoidSuccess) {
    auto a = make_success<void>();
    auto b = a.fmap<int>([]() { return make_success<int>(6); });

    BOOST_CHECK(a.valid());
    BOOST_CHECK_EQUAL(6, b.get());
}

BOOST_AUTO_TEST_CASE(MustFMapVoidSuccessToVoid) {
    auto a = make_success<void>();
    auto b = a.fmap<void>([]() { return make_success<void>(); });

    BOOST_CHECK(a.valid());
    BOOST_CHECK_NO_THROW(b.get());
}

BOOST_AUTO_TEST_CASE(MustFMapVoidFailure) {
    auto a = make_failure<void>(custom_exception("something went wrong"));
    auto b = a.fmap<int>([]() { return make_success<int>(6); });

    BOOST_CHECK(a.valid());
    BOOST_CHECK_THROW(b.get(), custom_exception);
}

BOOST_AUTO_TEST_CASE(MustFMapVoidFailureToVoid) {
    auto a = make_failure<void>(custom_exception("something went wrong"));
    auto b = a.fmap<void>([]() { return make_success<void>(); });

    BOOST_CHECK(a.valid());
    BOOST_CHECK_THROW(b.get(), custom_exception);
}

BOOST_AUTO_TEST_SUITE_END()

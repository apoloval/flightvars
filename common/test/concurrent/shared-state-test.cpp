/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <boost/test/unit_test.hpp>

#include <flightvars/concurrent/shared_state.hpp>

using namespace flightvars;
using namespace flightvars::concurrent;

BOOST_AUTO_TEST_SUITE(ConcurrentSharedState)

FV_DECL_EXCEPTION(custom_exception);

BOOST_AUTO_TEST_CASE(MustInvokeHandlerOnPush) {
    shared_state<std::string> s;
    auto result = util::make_none<std::string>();
    s.set_push_handler([&](util::attempt<std::string> r) {
        result = util::make_some(r.extract());
    });
    BOOST_CHECK(!result.is_defined());
    s.push_success("Hello!");
    BOOST_CHECK_EQUAL("Hello!", result.get());
}

BOOST_AUTO_TEST_CASE(MustInvokeHandlerSetAfterPush) {
    shared_state<std::string> s;
    s.push_success("Hello!");

    auto result = util::make_none<std::string>();
    s.set_push_handler([&](util::attempt<std::string> r) {
        result = util::make_some(r.extract());
    });
    BOOST_CHECK_EQUAL("Hello!", result.get());
}

BOOST_AUTO_TEST_CASE(MustShareStateOnCopy) {
    shared_state<std::string> s1;
    s1.push_success("Hello!");
    auto s2 = s1;

    auto result = util::make_none<std::string>();
    s2.set_push_handler([&](util::attempt<std::string> r) {
        result = util::make_some(r.extract());
    });
    BOOST_CHECK_EQUAL("Hello!", result.get());
}

BOOST_AUTO_TEST_CASE(MustInvalidateSourceOnMove) {
    shared_state<std::string> s1;
    auto s2 = std::move(s1);
    BOOST_CHECK(!s1.valid());
    BOOST_CHECK(s2.valid());
}

BOOST_AUTO_TEST_CASE(MustReset) {
    shared_state<std::string> s;
    s.reset();
    BOOST_CHECK(!s.valid());
    BOOST_CHECK_THROW(s.set_push_handler([](util::attempt<std::string>) {}), bad_shared_state);
    BOOST_CHECK_THROW(s.push_success("Hello!"), bad_shared_state);
}

BOOST_AUTO_TEST_CASE(MustPushSuccessForVoid) {
    shared_state<void> s;
    bool was_pushed = false;
    s.set_push_handler([&](util::attempt<void> r) {
        was_pushed = true;
    });
    BOOST_CHECK(!was_pushed);
    s.push_success();
    BOOST_CHECK(was_pushed);
}

BOOST_AUTO_TEST_SUITE_END()

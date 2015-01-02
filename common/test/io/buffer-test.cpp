/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#include <boost/test/unit_test.hpp>

#include <flightvars/io/buffer.hpp>

using namespace flightvars::io;

BOOST_AUTO_TEST_SUITE(IoBuffer)

BOOST_AUTO_TEST_CASE(MustInitWithExpectedPointers) {
    buffer buff(64);
    BOOST_CHECK_EQUAL(64, buff.size());
    BOOST_CHECK_EQUAL(64, buff.limit());
    BOOST_CHECK_EQUAL(0, buff.pos());
    BOOST_CHECK_EQUAL(64, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustInitWithInitializationList) {
    buffer buff({ 1, 2, 3 });
    BOOST_CHECK_EQUAL(3, buff.size());
    BOOST_CHECK_EQUAL(3, buff.limit());
    BOOST_CHECK_EQUAL(0, buff.pos());
    BOOST_CHECK_EQUAL(3, buff.remaining());

    BOOST_CHECK_EQUAL(1, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(2, buff.safe_read_value<std::uint8_t>());
    BOOST_CHECK_EQUAL(3, buff.safe_read_value<std::uint8_t>());
}

BOOST_AUTO_TEST_CASE(MustWriteDataOnAvailableSpace) {
    buffer buff(64);
    BOOST_CHECK_EQUAL(7, buff.write("hello!", 7));
    BOOST_CHECK_EQUAL(64, buff.size());
    BOOST_CHECK_EQUAL(64, buff.limit());
    BOOST_CHECK_EQUAL(7, buff.pos());
    BOOST_CHECK_EQUAL(57, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustWriteDataOnNonZeroPos) {
    buffer buff(64);
    BOOST_CHECK_EQUAL(6, buff.write("hello ", 6));
    BOOST_CHECK_EQUAL(7, buff.write("world!", 7));
    BOOST_CHECK_EQUAL(64, buff.size());
    BOOST_CHECK_EQUAL(64, buff.limit());
    BOOST_CHECK_EQUAL(13, buff.pos());
    BOOST_CHECK_EQUAL(51, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustWriteSomeDataOnSomeRemaining) {
    buffer buff(6);
    BOOST_CHECK_EQUAL(6, buff.write("hello!", 7));
    BOOST_CHECK_EQUAL(6, buff.size());
    BOOST_CHECK_EQUAL(6, buff.limit());
    BOOST_CHECK_EQUAL(6, buff.pos());
    BOOST_CHECK_EQUAL(0, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustNotWriteAnyDataOnNoneRemaining) {
    buffer buff(6);
    BOOST_CHECK_EQUAL(6, buff.write("hello", 6));
    BOOST_CHECK_EQUAL(0, buff.write("world!", 7));
    BOOST_CHECK_EQUAL(6, buff.size());
    BOOST_CHECK_EQUAL(6, buff.limit());
    BOOST_CHECK_EQUAL(6, buff.pos());
    BOOST_CHECK_EQUAL(0, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustHonourReset) {
    buffer buff(64);
    BOOST_CHECK_EQUAL(7, buff.write("hello!", 7));

    buff.reset();
    BOOST_CHECK_EQUAL(64, buff.size());
    BOOST_CHECK_EQUAL(64, buff.limit());
    BOOST_CHECK_EQUAL(0, buff.pos());
    BOOST_CHECK_EQUAL(64, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustHonourFlip) {
    buffer buff(64);
    BOOST_CHECK_EQUAL(7, buff.write("hello!", 7));

    buff.flip();
    BOOST_CHECK_EQUAL(64, buff.size());
    BOOST_CHECK_EQUAL(7, buff.limit());
    BOOST_CHECK_EQUAL(0, buff.pos());
    BOOST_CHECK_EQUAL(7, buff.remaining());
}

BOOST_AUTO_TEST_CASE(MustReadOnRemainingData) {
    buffer buff(64);
    BOOST_CHECK_EQUAL(7, buff.write("hello!", 7));

    buff.flip();
    char dest[64];
    BOOST_CHECK_EQUAL(7, buff.read(dest, 7));
    BOOST_CHECK_EQUAL(64, buff.size());
    BOOST_CHECK_EQUAL(7, buff.limit());
    BOOST_CHECK_EQUAL(7, buff.pos());
    BOOST_CHECK_EQUAL(0, buff.remaining());
    BOOST_CHECK_EQUAL("hello!", dest);
}

BOOST_AUTO_TEST_CASE(MustReadSomeOnSomeRemaining) {
    buffer buff(64);
    BOOST_CHECK_EQUAL(7, buff.write("hello!", 7));

    buff.flip();
    char dest[64];
    BOOST_CHECK_EQUAL(7, buff.read(dest, 64));
    BOOST_CHECK_EQUAL(64, buff.size());
    BOOST_CHECK_EQUAL(7, buff.limit());
    BOOST_CHECK_EQUAL(7, buff.pos());
    BOOST_CHECK_EQUAL(0, buff.remaining());
    BOOST_CHECK_EQUAL("hello!", dest);
}

BOOST_AUTO_TEST_CASE(MustReadNoneOnNoneRemaining) {
    buffer buff(64);
    BOOST_CHECK_EQUAL(7, buff.write("hello!", 7));

    buff.flip();
    char dest[64];
    BOOST_CHECK_EQUAL(7, buff.read(dest, 7));
    BOOST_CHECK_EQUAL(0, buff.read(dest, 7));
    BOOST_CHECK_EQUAL(64, buff.size());
    BOOST_CHECK_EQUAL(7, buff.limit());
    BOOST_CHECK_EQUAL(7, buff.pos());
    BOOST_CHECK_EQUAL(0, buff.remaining());
    BOOST_CHECK_EQUAL("hello!", dest);
}

BOOST_AUTO_TEST_CASE(MustWriteValue) {
    buffer buff(64);
    struct value { int a; float b; } val;
    val.a = 7;
    val.b = 5.5f;

    BOOST_CHECK_EQUAL(sizeof(value), buff.write_value(val));
    buff.reset();
    BOOST_CHECK_EQUAL(7, ((value*) buff.data())->a);
    BOOST_CHECK_EQUAL(5.5f, ((value*) buff.data())->b);
}

BOOST_AUTO_TEST_CASE(MustReadValue) {
    buffer buff(64);
    struct value { int a; float b; } val1, val2;
    val1.a = 7;
    val1.b = 5.5f;

    BOOST_CHECK_EQUAL(sizeof(value), buff.write_value(val1));
    buff.flip();
    BOOST_CHECK_EQUAL(sizeof(value), buff.read_value(val2));
    BOOST_CHECK_EQUAL(7, val2.a);
    BOOST_CHECK_EQUAL(5.5f, val2.b);
}

BOOST_AUTO_TEST_CASE(MustGetFirst) {
    buffer buff({1, 2, 3});

    BOOST_CHECK_EQUAL(1, buff.first());
}

BOOST_AUTO_TEST_CASE(MustGetLast) {
    buffer buff({ 1, 2, 3 });

    BOOST_CHECK_EQUAL(3, buff.last());
}

BOOST_AUTO_TEST_CASE(MustFailToGetLastOnLimitZero) {
    buffer buff(64);
    buff.flip();

    BOOST_CHECK_THROW(buff.last(), buffer_overflow);
}

BOOST_AUTO_TEST_CASE(MustGetLastOptSome) {
    buffer buff({ 1, 2, 3 });
    auto last = buff.last_opt();

    BOOST_CHECK(last.is_defined());
    BOOST_CHECK_EQUAL(3, last.get());
}

BOOST_AUTO_TEST_CASE(MustGetLastOptNoneOnLimitZero) {
    buffer buff(64);
    buff.flip();
    auto last = buff.last_opt();

    BOOST_CHECK(!last.is_defined());
}

BOOST_AUTO_TEST_SUITE_END()

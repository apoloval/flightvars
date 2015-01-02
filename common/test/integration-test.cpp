/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#define BOOST_TEST_MODULE "Integration Tests for FlightVars"
#include <boost/test/included/unit_test.hpp>

#include <flightvars/util/logging.hpp>

struct global_fixture {

    global_fixture() {
        flightvars::util::setup_console_logging();
    }
};

BOOST_GLOBAL_FIXTURE(global_fixture);

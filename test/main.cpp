/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#define BOOST_TEST_DYN_LINK
#define BOOST_TEST_MODULE FlightVars
#include <boost/test/unit_test.hpp>

#include <flightvars/util/logging.hpp>

struct global_fixture {

    global_fixture() {
        flightvars::util::setup_console_logging();
    }
};

BOOST_GLOBAL_FIXTURE(global_fixture);

#include "concurrent/future-test.hpp"
#include "concurrent/promise-test.hpp"
#include "io/buffer-test.hpp"
#include "io/tcp-server-it.hpp"
#include "mqtt/decoder/connect-test.hpp"
#include "mqtt/decoder/fixed-header-test.hpp"
#include "mqtt/decoder/types-test.hpp"
#include "mqtt/encoder/connect-test.hpp"
#include "mqtt/encoder/fixed-header-test.hpp"
#include "mqtt/encoder/types-test.hpp"
#include "mqtt/session-test.hpp"
#include "util/attempt-test.hpp"
#include "util/option-test.hpp"

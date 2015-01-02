/*
 * FlightVars
 * Copyright (c) 2014, 2015 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_IO_TYPES_H
#define FLIGHTVARS_IO_TYPES_H

#include <memory>

#include <boost/asio.hpp>

namespace flightvars { namespace io {

using tcp = boost::asio::ip::tcp;
using endpoint = tcp::endpoint;

using shared_socket = std::shared_ptr<tcp::socket>;

}}

#endif

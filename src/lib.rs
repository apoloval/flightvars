
//
// FlightVars
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[macro_use]
extern crate log;

extern crate libc;
extern crate log4rs;

// Only fs module is using `logging`.
// Remove this conditional compilation when that's not true.
#[cfg(windows)]
mod logging;

#[cfg(windows)]
mod fs;

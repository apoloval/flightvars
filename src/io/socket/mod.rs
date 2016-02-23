
//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;


pub trait Socket {
    /// The type of the input port of this socket
    type Input: io::Read;

    /// The type of the output port of this socket
    type Output: io::Write;

    /// Returns the input port of the socket
    fn input(&self) -> &Self::Input;

    /// Returns the output port of the socket
    fn output(&self) -> &Self::Output;
}

#[cfg(unix)]
mod unix;

#[cfg(unix)]
pub use self::unix::*;

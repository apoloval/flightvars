//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! This module defines the traits used to implement a communication transport layer.
//!
//! Communication in Rust is very hard to implement if you are not following the standard
//! use case. And FlightVars has a communication pattern that is not standard at all. There
//! are two factors that complicate the situation:
//!
//! * The communication pattern. FlightVars doesn't follow a client-server communication model.
//!   Both client and server may initiate a communication to each other at any time. Thus, for
//!   sync IO two threads per endpoint must be provided: one for reading and one for writing. That
//!   means the comm device must be shared by two different threads. Rust prevents resource sharing
//!   among threads by design. So we have a problem here.
//!
//! * The termination pattern. FlightVars must run as a plugin in FSX/Prepar3D. That means it
//!   have to be started upon sim launch and stopped when sim is shutdown. All classes in Rust
//!   standard library `std::net` assume that servers runs forever in a infinite loop. In
//!   FlightVars, the ability to stop the transport layer is required. We have another problem
//!   here.
//!
//! This module provides the traits aimed to implement a transport layer that solves the two
//! problems identified above. The communication pattern problem is solved by ensuring the
//! transport is able to split read and write endpoints for each new connection. Each one will be
//! send to a different thread when transport is used. The termination pattern problem is solved
//! by requiring listeners to provide a handle to be interrupted. This handle may be used from 
//! another thread to indicate the listener must terminate. Also readers, which are by nature
//! blocking, are guaranteed to be configured with timeouts. If there is no incoming data after
//! some duration, the read operation fails with `std::io::ErrorKind::TimedOut` error kind. 
//!
//! # Traits
//!
//! * `Interrupt`, which represents a handle to interrupt a process
//! * `Listen`, which represents a instance that can be used to listen for a pair of input and
//!   output endpoints. It provides the meanings to obtain an interruption handle. 
//! * `Transport`, which represents a instance to bind everything else.

use std::fmt;
use std::io;

pub mod serial;
pub mod tcp;

/// An object able to interrupt a communication process.
///
/// See [the module level documentation](index.html) for more.
pub trait Interrupt {

    fn interrupt(self);
}

/// An object able to listen for incoming connections.
///
/// It produces a tuple of connection address, input and output endpoints ready to be used from 
/// different threads. See [the module level documentation](index.html) for more.
pub trait Listen {
	type ConnAddr: fmt::Display;
	type Input: io::Read;
	type Output: io::Write;
    type Int: Interrupt;
	
    fn listen(&mut self) -> io::Result<(Self::Input, Self::Output, Self::ConnAddr)>;

    fn shutdown_interruption(&mut self) -> Self::Int;
}

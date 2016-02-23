
//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

use super::Socket;

pub struct UnixSocketInput;

impl io::Read for UnixSocketInput {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> { unimplemented!() }
}

pub struct UnixSocketOutput;

impl io::Write for UnixSocketOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> { unimplemented!() }
    fn flush(&mut self) -> io::Result<()> { unimplemented!() }
}

pub struct UnixSocket;

impl Socket for UnixSocket {
    type Input = UnixSocketInput;

    /// The type of the output port of this socket
    type Output = UnixSocketOutput;

    /// Returns the input port of the socket
    fn input(&self) -> &UnixSocketInput { unimplemented!() }

    /// Returns the output port of the socket
    fn output(&self) -> &UnixSocketOutput { unimplemented!() }
}

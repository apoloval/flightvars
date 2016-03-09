//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

pub mod tcp;

pub trait ShutdownHandle {
    fn shutdown(self);
}

pub trait Transport {
    type Input;
    type Output;
    type Shutdown : ShutdownHandle;

    fn wait_conn(&mut self) -> io::Result<(Self::Input, Self::Output)>;

    fn shutdown_handle(&mut self) -> Self::Shutdown;
}

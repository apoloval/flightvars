//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

pub mod tcp;

pub trait Interrupt {
    fn interrupt(self);
}

pub trait ShutdownInterruption {
    type Int: Interrupt;
    fn shutdown_interruption(&mut self) -> Self::Int;
}

pub trait Listen<I, O> {
    fn listen(&mut self) -> io::Result<(I, O)>;
}

pub trait Transport {
    type Input: ShutdownInterruption;
    type Output;
    type Listener: Listen<Self::Input, Self::Output> + ShutdownInterruption;

    fn listener(&mut self) -> &mut Self::Listener;
}

//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::sync::mpsc;

use domain::{Command, Event};

pub mod oacsp;
pub mod dummy;

pub trait MessageRead {
    fn read_msg(&mut self) -> io::Result<Command>;
}


pub trait MessageWrite {
    fn write_msg(&mut self, msg: &Event) -> io::Result<()>;
}


pub trait Protocol<I, O> {
    type Read: MessageRead;
    type Write: MessageWrite;
    fn reader(&self, input: I) -> Self::Read;
    fn writer(&self, output: O) -> Self::Write;
}

pub trait BidirProtocol<T> : Protocol<T, T> {}

impl<T, P: Protocol<T, T>> BidirProtocol<T> for P {}

pub fn oacsp() -> oacsp::Oacsp { oacsp::Oacsp }
pub fn dummy() -> dummy::DummyProtocol { dummy::DummyProtocol }

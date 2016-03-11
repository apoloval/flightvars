//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::sync::mpsc;

pub mod oacsp;

#[derive(Clone, Debug, PartialEq)]
pub enum Message<F> {
    Open,
    WriteData(i32, F)
}

pub type RawMessage = Message<()>;

impl RawMessage {
    pub fn map_origin(&self, origin: &mpsc::Sender<RawMessage>) -> MessageFrom {
        match self {
            &Message::Open => Message::Open,
            &Message::WriteData(d, _) => Message::WriteData(d, origin.clone())
        }
    }
}

pub type MessageFrom = Message<mpsc::Sender<RawMessage>>;

pub trait MessageRead {
    fn read_msg(&mut self) -> io::Result<RawMessage>;
}

pub trait MessageWrite {
    fn write_msg<F>(&mut self, msg: &Message<F>) -> io::Result<()>;
}

pub trait Protocol<R: io::Read, W: io::Write> {
    type Read: MessageRead;
    type Write: MessageWrite;
    fn reader(&self, input: R) -> Self::Read;
    fn writer(&self, output: W) -> Self::Write;
}

pub trait BidirProtocol<T: io::Read + io::Write> : Protocol<T, T> {}

impl<T: io::Read + io::Write, P: Protocol<T, T>> BidirProtocol<T> for P {}

pub fn oacsp() -> oacsp::Oacsp { oacsp::Oacsp }

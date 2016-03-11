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

pub trait Protocol<R: io::Read> {
    type Decoder: Iterator<Item=RawMessage>;
    fn decode(&self, input: R) -> Self::Decoder;
}

pub fn oacsp() -> oacsp::Oacsp { oacsp::Oacsp }

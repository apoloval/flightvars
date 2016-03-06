//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

pub mod oacsp;

pub enum Message {
    Open
}

pub trait ProtocolRead<R: io::Read>: Iterator<Item=io::Result<Message>> {
    fn from(input: R) -> Self;
}

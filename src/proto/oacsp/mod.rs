//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

use proto::*;

mod msg;

pub struct Oacsp<R: io::Read> {
    msg_input: msg::MessageIter<R>
}

impl<R: io::Read> ProtocolRead<R> for Oacsp<R> {
    fn from(input: R) -> Oacsp<R> {
        Oacsp { msg_input: msg::MessageIter::new(input) }
    }
}

impl<R: io::Read> Iterator for Oacsp<R> {
    type Item = io::Result<Message>;

    fn next(&mut self) -> Option<io::Result<Message>> {
        match self.msg_input.next() {
            _ => None,
        }
    }
}

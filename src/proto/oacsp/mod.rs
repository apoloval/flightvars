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

pub struct Oacsp;

impl<R: io::Read> ProtocolRead<R> for Oacsp {
    type IntoIter = OacspIter<R>;
    fn iter_from(&self, input: R) -> OacspIter<R> {
        OacspIter { msg_input: msg::MessageIter::new(input) }
    }
}

struct OacspIter<R: io::Read> {
    msg_input: msg::MessageIter<R>
}

unsafe impl<R: io::Read> Send for OacspIter<R> {}

impl<R: io::Read> Iterator for OacspIter<R> {
    type Item = Message;

    fn next(&mut self) -> Option<Message> {
        match self.msg_input.next() {
            _ => None,
        }
    }
}

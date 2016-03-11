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

#[derive(Clone)]
pub struct Oacsp;

impl<R: io::Read> Protocol<R> for Oacsp {
    type Decoder = OacspDecoder<R>;
    fn decode(&self, input: R) -> OacspDecoder<R> {
        OacspDecoder { msg_input: msg::MessageIter::new(input) }
    }
}

struct OacspDecoder<R: io::Read> {
    msg_input: msg::MessageIter<R>
}

impl<R: io::Read> Iterator for OacspDecoder<R> {
    type Item = RawMessage;

    fn next(&mut self) -> Option<RawMessage> {
        match self.msg_input.next() {
            _ => None,
        }
    }
}

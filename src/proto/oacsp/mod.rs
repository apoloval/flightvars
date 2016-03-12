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

impl<R: io::Read, W: io::Write> Protocol<R, W> for Oacsp {
    type Read = OacspReader<R>;
    type Write = OacspWriter<W>;

    fn reader(&self, input: R) -> OacspReader<R> {
        OacspReader { input: io::BufReader::new(input) }
    }

    fn writer(&self, output: W) ->OacspWriter<W> {
        OacspWriter { output: output }
    }
}

pub struct OacspReader<R: io::Read> {
    input: io::BufReader<R>
}

impl<R: io::Read> MessageRead for OacspReader<R> {
    fn read_msg(&mut self) -> io::Result<RawMessage> {
        use std::io::BufRead;
        use self::msg::*;
        let mut line = String::new();
        let msg: io::Result<msg::Message> = match self.input.read_line(&mut line) {
            Ok(_) => line.trim().parse(),
            Err(e) => Err(e),
        };
        unimplemented!()
    }
}

pub struct OacspWriter<W: io::Write> {
    output: W
}

impl<W: io::Write> MessageWrite for OacspWriter<W> {
    fn write_msg(&mut self, msg: &RawMessage) -> io::Result<()> {
        unimplemented!()
    }
}

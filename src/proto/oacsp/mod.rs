//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

use proto::*;

mod input;
//mod output;
//mod reader;
//mod writer;

//use self::input::*;
//use self::output::*;
//pub use self::reader::*;
//pub use self::writer::*;

pub struct Oacsp;

impl Protocol for Oacsp {
    
    fn decode<R: io::Read>(&mut self, input: &R) -> io::Result<InputMessage> {
        unimplemented!()
    }
    
    fn encode<W: io::Write>(&mut self, message: &OutputMessage, output: &W) -> io::Result<()> {
        unimplemented!()
    }
    
}


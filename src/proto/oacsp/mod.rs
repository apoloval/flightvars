//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

use domain::Client;
use proto::*;

mod input;
mod output;
mod reader;
mod writer;

use self::input::*;
use self::output::*;
pub use self::reader::*;
pub use self::writer::*;


#[derive(Clone)]
pub struct Oacsp;

impl<R: io::Read, W: io::Write> Protocol<R, W> for Oacsp {
    type Read = CommandReader<MessageIter<R>>;
    type Write = EventWriter<MessageConsumer<W>>;
    
    fn name(&self) -> &str { "oacsp" }

    fn reader(&self, input: R, id: Client) -> CommandReader<MessageIter<R>> {
        CommandReader::new(MessageIter::new(input), id)
    }

    fn writer(&self, output: W) -> EventWriter<MessageConsumer<W>> {
        EventWriter::new(MessageConsumer::new(output))
    }
}

//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

use domain::{Client, Command, Event};
use proto::*;

mod msg;

#[derive(Clone)]
pub struct Oacsp;

impl<R: io::Read, W: io::Write> Protocol<R, W> for Oacsp {
    type Read = CommandReader<R>;
    type Write = EventWriter<W>;

    fn reader(&self, input: R, id: Client) -> CommandReader<R> {
        CommandReader { input: io::BufReader::new(input), id: id }
    }

    fn writer(&self, output: W) -> EventWriter<W> {
        EventWriter { output: output }
    }
}

pub struct CommandReader<R: io::Read> {
    input: io::BufReader<R>,
    id: Client,
}

impl<R: io::Read> CommandRead for CommandReader<R> {
    fn read_cmd(&mut self) -> io::Result<Command> {
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

pub struct EventWriter<W: io::Write> {
    output: W
}

impl<W: io::Write> EventWrite for EventWriter<W> {
    fn write_ev(&mut self, msg: &Event) -> io::Result<()> {
        unimplemented!()
    }
}

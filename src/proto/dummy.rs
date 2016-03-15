//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

use comm::dummy;
use domain::*;
use proto;

#[derive(Clone, Debug, PartialEq)]
pub enum DummyCommand {
    Observe(Domain, Var),
    Write(Domain, Var, Value),
}

impl DummyCommand {
    pub fn into_cmd(self, id: Client) -> Command {
        match self {
            DummyCommand::Observe(d, v) => Command::Observe(d, v, id),
            DummyCommand::Write(d, var, val) => Command::Write(d, var, val),
        }
    }
}

impl From<Command> for DummyCommand {
    fn from(cmd: Command) -> DummyCommand {
        match cmd {
            Command::Observe(d, v, _) => DummyCommand::Observe(d, v),
            Command::Write(d, var, val) => DummyCommand::Write(d, var, val),
        }
    }
}

pub struct DummyProtocol;

pub type DummyProtocolInput = dummy::DummyTransportInput<DummyCommand>;
pub type DummyProtocolOutput = dummy::DummyTransportOutput<Event>;

impl proto::Protocol<DummyProtocolInput, DummyProtocolOutput> for DummyProtocol {
    type Read = CommandReader;
    type Write = EventWriter;

    fn reader(&self, input: DummyProtocolInput, id: Client) -> Self::Read {
        CommandReader { input: input, id: id }
    }

    fn writer(&self, output: DummyProtocolOutput) -> Self::Write {
        EventWriter { output: output }
    }
}

pub struct CommandReader {
    input: DummyProtocolInput,
    id: Client,
}

impl proto::CommandRead for CommandReader {
    fn read_cmd(&mut self) -> io::Result<Command> {
        match self.input.recv() {
            dummy::StreamEvent::Message(msg) => Ok(msg.into_cmd(self.id.clone())),
            dummy::StreamEvent::Shutdown => Err(io::Error::new(
                io::ErrorKind::Interrupted, "connection interrupted")),
        }
    }
}

pub struct EventWriter {
    output: DummyProtocolOutput
}

impl proto::EventWrite for EventWriter {
    fn write_ev(&mut self, msg: &Event) -> io::Result<()> {
        self.output.send(msg.clone());
        Ok(())
    }
}

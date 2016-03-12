//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::sync::mpsc;

use comm::dummy;
use proto;

pub struct DummyProtocol;

pub type DummyProtocolInput = dummy::DummyTransportInput<proto::RawMessage>;
pub type DummyProtocolOutput = dummy::DummyTransportOutput<proto::RawMessage>;

impl proto::Protocol<DummyProtocolInput, DummyProtocolOutput> for DummyProtocol {
    type Read = MessageReader;
    type Write = MessageWriter;

    fn reader(&self, input: DummyProtocolInput) -> Self::Read {
        MessageReader { input: input }
    }

    fn writer(&self, output: DummyProtocolOutput) -> Self::Write {
        MessageWriter { output: output }
    }
}

pub struct MessageReader {
    input: DummyProtocolInput
}

impl proto::MessageRead for MessageReader {
    fn read_msg(&mut self) -> io::Result<proto::RawMessage> {
        match self.input.recv() {
            dummy::StreamEvent::Message(msg) => Ok(msg),
            dummy::StreamEvent::Shutdown => Err(io::Error::new(
                io::ErrorKind::Interrupted, "connection interrupted")),
        }
    }
}

pub struct MessageWriter {
    output: DummyProtocolOutput
}

impl proto::MessageWrite for MessageWriter {
    fn write_msg(&mut self, msg: &proto::RawMessage) -> io::Result<()> {
        self.output.send(msg.clone());
        Ok(())
    }
}

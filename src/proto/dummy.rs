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

pub type Stream = dummy::StreamEventChannel<proto::RawMessage>;

pub struct DummyProtocol;

impl proto::Protocol<Stream, Stream> for DummyProtocol {
    type Read = MessageReader;
    type Write = MessageWriter;

    fn reader(&self, input: Stream) -> Self::Read {
        MessageReader { rx: input.rx }
    }

    fn writer(&self, output: Stream) -> Self::Write {
        MessageWriter { tx: output.tx }
    }
}

pub struct MessageReader {
    rx: mpsc::Receiver<dummy::StreamEvent<proto::RawMessage>>
}

impl proto::MessageRead for MessageReader {
    fn read_msg(&mut self) -> io::Result<proto::RawMessage> {
        match self.rx.recv().unwrap() {
            dummy::StreamEvent::Message(msg) => Ok(msg),
            dummy::StreamEvent::Shutdown => Err(io::Error::new(
                io::ErrorKind::Interrupted, "connection interrupted")),
        }
    }
}

pub struct MessageWriter {
    tx: mpsc::Sender<dummy::StreamEvent<proto::RawMessage>>
}

impl proto::MessageWrite for MessageWriter {
    fn write_msg(&mut self, msg: &proto::RawMessage) -> io::Result<()> {
        self.tx.send(dummy::StreamEvent::Message(msg.clone())).unwrap();
        Ok(())
    }
}

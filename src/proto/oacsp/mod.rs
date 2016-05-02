//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

use domain::{Client, Event, Var};
use proto::*;
use util::Consume;

mod msg;
mod reader;

use self::msg::*;
pub use self::reader::*;

#[derive(Clone)]
pub struct Oacsp;

impl<R: io::Read, W: io::Write> Protocol<R, W> for Oacsp {
    type Read = CommandReader<MessageIter<R>>;
    type Write = EventWriter<MessageConsumer<W>>;

    fn reader(&self, input: R, id: Client) -> CommandReader<MessageIter<R>> {
        CommandReader::new(MessageIter::new(input), id)
    }

    fn writer(&self, output: W) -> EventWriter<MessageConsumer<W>> {
        EventWriter::new(MessageConsumer::new(output))
    }
}


pub struct EventWriter<C: Consume<Item=OutputMessage, Error=io::Error>> {
    consumer: C
}

impl<C: Consume<Item=OutputMessage, Error=io::Error>> EventWriter<C> {
    pub fn new(consumer: C) -> EventWriter<C> {
        EventWriter { consumer: consumer }
    }
}

impl<C: Consume<Item=OutputMessage, Error=io::Error>> EventWrite for EventWriter<C> {
    fn write_ev(&mut self, ev: &Event) -> io::Result<()> {
        match ev {
            &Event::Update(Var::LVar(ref name), ref value) =>
                self.consumer.consume(OutputMessage::EventLvar {
                    lvar: name.clone(),
                    value: value.clone()
                }),
            &Event::Update(Var::FsuipcOffset(ref offset), ref value) =>
                self.consumer.consume(OutputMessage::EventOffset {
                    offset: offset.addr().clone(),
                    value: value.clone()
                }),
            msg => Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("cannot write event {:?} using oacsp protocol", msg))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::mpsc;

    use domain::types::*;
    use domain::fsuipc::types::*;
    use proto::*;

    use super::*;
    use super::msg::*;    

    #[test]
    fn should_consume_lvar_event() {
        let (tx, rx) = mpsc::channel();
        let mut writer = EventWriter::new(tx);
        writer.write_ev(&Event::Update(Var::lvar("lvar"), Value::Int(42))).unwrap();
        let actual = rx.recv().unwrap();
        let expected = OutputMessage::event_lvar("lvar", Value::Int(42));
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_consume_fsuipc_offset_event() {
        let (tx, rx) = mpsc::channel();
        let mut writer = EventWriter::new(tx);
        writer.write_ev(&Event::Update(
            Var::FsuipcOffset(Offset::new(OffsetAddr::from(0x1234), OffsetLen::UnsignedWord)),
            Value::UnsignedInt(42))).unwrap();
        let actual = rx.recv().unwrap();
        let expected = OutputMessage::event_offset(
            OffsetAddr::from(0x1234), Value::UnsignedInt(42));
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_fail_consume_close_event() {
        let (tx, _rx) = mpsc::channel();
        let mut writer = EventWriter::new(tx);
        let err = writer.write_ev(&Event::Close).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }
}

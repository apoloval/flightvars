//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

use domain;
use domain::{Client, Command, Event};
use proto::*;
use util::Consume;

mod msg;
use self::msg::*;

#[derive(Clone)]
pub struct Oacsp;

impl<R: io::Read, W: io::Write> Protocol<R, W> for Oacsp {
    type Read = CommandReader<MessageIter<R>>;
    type Write = EventWriter<MessageConsumer<W>>;

    fn reader(&self, input: R, id: Client) -> CommandReader<MessageIter<R>> {
        CommandReader::new(MessageIter::new(input), id)
    }

    fn writer(&self, output: W) -> EventWriter<MessageConsumer<W>> {
        EventWriter { consumer: MessageConsumer::new(output) }
    }
}

pub struct CommandReader<I: Iterator<Item=io::Result<InputMessage>>> {
    input: I,
    id: Client,
    ready: bool,
}

impl<I> CommandReader<I> where I: Iterator<Item=io::Result<InputMessage>> {
    pub fn new(input: I, id: Client) -> CommandReader<I> {
        CommandReader { input: input, id: id, ready: false }
    }

    fn require_ready(&self) -> io::Result<()> {
        if self.ready { Ok(()) }
        else { Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected a begin message, but other was received"))
        }
    }
}

impl<I: Iterator<Item=io::Result<InputMessage>>> CommandRead for CommandReader<I> {
    fn read_cmd(&mut self) -> io::Result<Command> {
        let msg = try!(try!(self.input.next().ok_or_else(||
            io::Error::new(io::ErrorKind::ConnectionReset,
            "input stream was closed"))));
        match msg {
            InputMessage::Begin { version, client_id } => {
                info!("oacsp client v{} connected with ID {}", version, client_id);
                self.ready = true;
                self.read_cmd()
            },
            InputMessage::WriteLvar { lvar, value } => {
                try!(self.require_ready());
                Ok(Command::Write(domain::Var::LVar(lvar), value))
            },
            InputMessage::WriteOffset { offset, value } => {
                try!(self.require_ready());
                Ok(Command::Write(domain::Var::FsuipcOffset(offset), value))
            },
            InputMessage::ObserveLvar { lvar } => {
                try!(self.require_ready());
                Ok(Command::Observe(domain::Var::LVar(lvar), self.id.clone()))
            },
            InputMessage::ObserveOffset { offset } => {
                try!(self.require_ready());
                Ok(Command::Observe(domain::Var::FsuipcOffset(offset), self.id.clone()))
            },
        }
    }
}

pub struct EventWriter<C: Consume<OutputMessage>> {
    consumer: C
}

impl<C: Consume<OutputMessage>> EventWrite for EventWriter<C> {
    fn write_ev(&mut self, msg: &Event) -> io::Result<()> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::mpsc;

    use domain;
    use domain::types::*;
    use domain::fsuipc::types::*;
    use proto::*;

    use super::*;
    use super::msg::*;

    #[test]
    fn should_reject_msg_before_begin() {
        let (tx, _) = mpsc::channel();
        let id = Client::new("client", tx);
        let input: Vec<io::Result<InputMessage>> = vec![
            Ok(InputMessage::write_lvar("lvar", Value::Int(42)))
        ];
        let mut reader = CommandReader::new(input.into_iter(), id);
        let err = reader.read_cmd().unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn should_process_write_lvar() {
        let (tx, _) = mpsc::channel();
        let id = Client::new("client", tx);
        let input: Vec<io::Result<InputMessage>> = vec![
            Ok(InputMessage::begin(1, "foobar")),
            Ok(InputMessage::write_lvar("lvar", Value::Int(42)))
        ];
        let mut reader = CommandReader::new(input.into_iter(), id);
        let actual = reader.read_cmd().unwrap();
        let expected = Command::Write(Var::lvar("lvar"), domain::Value::Int(42));
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_process_write_offset() {
        let (tx, _) = mpsc::channel();
        let id = Client::new("client", tx);
        let input: Vec<io::Result<InputMessage>> = vec![
            Ok(InputMessage::begin(1, "foobar")),
            Ok(InputMessage::write_offset(
                Offset::new(OffsetAddr::from(0x1234), OffsetLen::UnsignedWord),
                Value::UnsignedInt(42)))
        ];
        let mut reader = CommandReader::new(input.into_iter(), id);
        let actual = reader.read_cmd().unwrap();
        let expected = Command::Write(
            Var::FsuipcOffset(Offset::new(OffsetAddr::from(0x1234), OffsetLen::UnsignedWord)),
            Value::UnsignedInt(42));
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_process_obs_lvar() {
        let (tx, _) = mpsc::channel();
        let id = Client::new("client", tx);
        let input: Vec<io::Result<InputMessage>> = vec![
            Ok(InputMessage::begin(1, "foobar")),
            Ok(InputMessage::obs_lvar("lvar"))
        ];
        let mut reader = CommandReader::new(input.into_iter(), id.clone());
        let actual = reader.read_cmd().unwrap();
        let expected = Command::Observe(Var::lvar("lvar"), id);
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_process_obs_offset() {
        let (tx, _) = mpsc::channel();
        let id = Client::new("client", tx);
        let input: Vec<io::Result<InputMessage>> = vec![
            Ok(InputMessage::begin(1, "foobar")),
            Ok(InputMessage::obs_offset(
                Offset::new(OffsetAddr::from(0x1234), OffsetLen::UnsignedWord)))
        ];
        let mut reader = CommandReader::new(input.into_iter(), id.clone());
        let actual = reader.read_cmd().unwrap();
        let expected = Command::Observe(
            Var::FsuipcOffset(Offset::new(OffsetAddr::from(0x1234), OffsetLen::UnsignedWord)), id);
        assert_eq!(actual, expected);
    }

    #[test]
    fn should_write_lvar_event() {
    }
}

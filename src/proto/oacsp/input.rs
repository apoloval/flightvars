//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::str::FromStr;

use domain::types::*;
use domain::fsuipc::types::*;

#[derive(Debug, PartialEq)]
pub enum InputMessage {
    Begin { version: u16, client_id: String },
    WriteLvar { lvar: String, value: Value },
    WriteOffset { offset: Offset, value: Value },
    ObserveLvar { lvar: String },
    ObserveOffset { offset: Offset },
}

impl InputMessage {

    pub fn begin(version: u16, client_id: &str) -> InputMessage {
        InputMessage::Begin { version: version, client_id: client_id.to_string() }
    }

    pub fn write_lvar(lvar: &str, value: Value) -> InputMessage {
        InputMessage::WriteLvar { lvar: lvar.to_string(), value: value }
    }

    pub fn write_offset(offset: Offset, value: Value) -> InputMessage {
        InputMessage::WriteOffset { offset: offset, value: value }
    }

    pub fn obs_lvar(lvar: &str) -> InputMessage {
        InputMessage::ObserveLvar { lvar: lvar.to_string() }
    }

    pub fn obs_offset(offset: Offset) -> InputMessage {
        InputMessage::ObserveOffset { offset: offset }
    }
}

impl FromStr for InputMessage {
    type Err = io::Error;
    fn from_str(s: &str) -> Result<InputMessage, io::Error> {
        let deco = MessageParser::new(s);
        deco.parse()
    }
}


pub struct MessageParser<'a> {
    input: &'a str
}

impl<'a> MessageParser<'a> {
    pub fn new(input: &'a str) -> MessageParser {
        MessageParser { input: input }
    }

    pub fn parse(self) -> io::Result<InputMessage> {
        if self.input.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput, "cannot parse oacsp message from empty line"));
        }
        let args: Vec<&str> = self.input.split_whitespace().collect();
        let cmd = args[0];
        let args = &args[1..];
        match &cmd.to_uppercase()[..] {
            "BEGIN" => self.parse_begin(&args),
            "WRITE_LVAR" => self.parse_write_lvar(&args),
            "WRITE_OFFSET" => self.parse_write_offset(&args),
            "OBS_LVAR" => self.parse_obs_lvar(&args),
            "OBS_OFFSET" => self.parse_obs_offset(&args),
            _ => Err(self.input_error()),
        }
    }

    fn parse_begin(self, args: &[&str]) -> io::Result<InputMessage> {
        try!(self.require_argc(args, 2));
        let ver = try!(args[0].parse().map_err(|_| self.input_error()));
        Ok(InputMessage::begin(ver, args[1]))
    }

    fn parse_write_lvar(self, args: &[&str]) -> io::Result<InputMessage> {
        try!(self.require_argc(args, 2));
        let lvar = args[0];
        let value = try!(Value::parse_int(args[1]).map_err(|_| self.input_error()));
        Ok(InputMessage::write_lvar(&lvar, value))
    }

    fn parse_write_offset(self, args: &[&str]) -> io::Result<InputMessage> {
        try!(self.require_argc(args, 2));
        let offset: Offset = try!(args[0].parse());
        let value = try!(offset.parse_value(args[1]).map_err(|_| self.input_error()));
        Ok(InputMessage::write_offset(offset, value))
    }

    fn parse_obs_lvar(self, args: &[&str]) -> io::Result<InputMessage> {
        try!(self.require_argc(args, 1));
        Ok(InputMessage::obs_lvar(args[0]))
    }

    fn parse_obs_offset(self, args: &[&str]) -> io::Result<InputMessage> {
        try!(self.require_argc(args, 1));
        Ok(InputMessage::obs_offset(try!(args[0].parse())))
    }

    fn require_argc(&self, args: &[&str], expected: usize) -> io::Result<()> {
        if args.len() == expected { Ok(()) }
        else { Err(self.input_error()) }
    }

    fn input_error(&self) -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid oacsp message in '{}'", self.input))
    }
}

pub struct MessageIter<R: io::Read> {
    input: io::BufReader<R>
}

impl<R: io::Read> MessageIter<R> {
    pub fn new(input: R) -> MessageIter<R> {
        MessageIter { input: io::BufReader::new(input) }
    }
}

impl<R: io::Read> Iterator for MessageIter<R> {

    type Item = io::Result<InputMessage>;
    fn next(&mut self) -> Option<io::Result<InputMessage>> {
        use std::io::BufRead;
        let mut line = String::new();
        match self.input.read_line(&mut line) {
            Ok(0) => return None,
            Err(e) => return Some(Err(e)),
            _ => {},
        }
        Some(line.trim().parse())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use domain::types::*;
    use domain::fsuipc::types::*;

    use super::*;

    #[test]
    fn should_parse_begin_msg() {
        let buf = "BEGIN 1 arduino";
        let msg = InputMessage::from_str(&buf).unwrap();
        assert_eq!(msg, InputMessage::begin(1, "arduino"));
    }

    #[test]
    fn should_parse_write_lvar_msg() {
        let buf = "WRITE_LVAR foobar 42";
        let msg = InputMessage::from_str(&buf).unwrap();
        assert_eq!(msg, InputMessage::write_lvar("foobar", Value::Int(42)));
    }

    #[test]
    fn should_parse_write_offset_msg() {
        let buf = "WRITE_OFFSET 1234:UW 42";
        let msg = InputMessage::from_str(&buf).unwrap();
        assert_eq!(
            msg,
            InputMessage::write_offset(
                Offset::new(OffsetAddr::from(0x1234),
                OffsetLen::UnsignedWord),
                Value::UnsignedInt(42)));
    }

    #[test]
    fn should_parse_obs_lvar_msg() {
        let buf = "OBS_LVAR foobar";
        let msg = InputMessage::from_str(&buf).unwrap();
        assert_eq!(msg, InputMessage::obs_lvar("foobar"));
    }

    #[test]
    fn should_parse_obs_offset_msg() {
        let buf = "OBS_OFFSET 1234:UW";
        let msg = InputMessage::from_str(&buf).unwrap();
        assert_eq!(msg, InputMessage::obs_offset(
            Offset::new(OffsetAddr::from(0x1234), OffsetLen::UnsignedWord)));
    }

    #[test]
    fn should_fail_to_parse_empty_line() {
        let buf = "";
        assert!(InputMessage::from_str(&buf).is_err());
    }

    #[test]
    fn should_iterate_messages_from_read() {
        let buf = b"BEGIN 1 arduino\nOBS_LVAR foobar\nwrong line\n\nWRITE_LVAR foobar 42";
        let mut iter = MessageIter::new(&buf[..]);
        assert_eq!(iter.next().unwrap().unwrap(), InputMessage::begin(1, "arduino"));
        assert_eq!(iter.next().unwrap().unwrap(), InputMessage::obs_lvar("foobar"));
        assert!(iter.next().unwrap().is_err());
        assert!(iter.next().unwrap().is_err());
        assert_eq!(
            iter.next().unwrap().unwrap(),
            InputMessage::write_lvar("foobar", Value::Int(42)));
        assert!(iter.next().is_none());
    }
}

//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::str::FromStr;

use types::*;

#[derive(Debug, PartialEq)]
pub enum RawInputMessage {
    Begin { version: u16, client_id: String },
    WriteLvar { lvar: String, value: Value },
    WriteOffset { offset: Offset, value: Value },
    ObserveLvar { lvar: String },
    ObserveOffset { offset: Offset },
}

impl RawInputMessage {

    pub fn begin(version: u16, client_id: &str) -> RawInputMessage {
        RawInputMessage::Begin { version: version, client_id: client_id.to_string() }
    }

    pub fn write_lvar(lvar: &str, value: Value) -> RawInputMessage {
        RawInputMessage::WriteLvar { lvar: lvar.to_string(), value: value }
    }

    pub fn write_offset(offset: Offset, value: Value) -> RawInputMessage {
        RawInputMessage::WriteOffset { offset: offset, value: value }
    }

    pub fn obs_lvar(lvar: &str) -> RawInputMessage {
        RawInputMessage::ObserveLvar { lvar: lvar.to_string() }
    }

    pub fn obs_offset(offset: Offset) -> RawInputMessage {
        RawInputMessage::ObserveOffset { offset: offset }
    }
}

impl FromStr for RawInputMessage {
    type Err = io::Error;
    fn from_str(s: &str) -> io::Result<RawInputMessage> {
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

    pub fn parse(self) -> io::Result<RawInputMessage> {
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

    fn parse_begin(self, args: &[&str]) -> io::Result<RawInputMessage> {
        try!(self.require_argc(args, 2));
        let ver = try!(args[0].parse().map_err(|_| self.input_error()));
        Ok(RawInputMessage::begin(ver, args[1]))
    }

    fn parse_write_lvar(self, args: &[&str]) -> io::Result<RawInputMessage> {
        try!(self.require_argc(args, 2));
        let lvar = args[0];
        let value = try!(args[1].parse().map(Value::Number).map_err(|_| self.input_error()));
        Ok(RawInputMessage::write_lvar(&lvar, value))
    }

    fn parse_write_offset(self, args: &[&str]) -> io::Result<RawInputMessage> {
        try!(self.require_argc(args, 2));
        let offset: Offset = try!(args[0].parse());
        let value = try!(args[1].parse().map(Value::Number).map_err(|_| self.input_error()));
        Ok(RawInputMessage::write_offset(offset, value))
    }

    fn parse_obs_lvar(self, args: &[&str]) -> io::Result<RawInputMessage> {
        try!(self.require_argc(args, 1));
        Ok(RawInputMessage::obs_lvar(args[0]))
    }

    fn parse_obs_offset(self, args: &[&str]) -> io::Result<RawInputMessage> {
        try!(self.require_argc(args, 1));
        Ok(RawInputMessage::obs_offset(try!(args[0].parse())))
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use types::*;

    use super::*;

    #[test]
    fn should_parse_begin_msg() {
        let buf = "BEGIN 1 arduino";
        let msg = RawInputMessage::from_str(&buf).unwrap();
        assert_eq!(msg, RawInputMessage::begin(1, "arduino"));
    }

    #[test]
    fn should_parse_write_lvar_msg() {
        let buf = "WRITE_LVAR foobar 42";
        let msg = RawInputMessage::from_str(&buf).unwrap();
        assert_eq!(msg, RawInputMessage::write_lvar("foobar", Value::Number(42)));
    }

    #[test]
    fn should_parse_write_offset_msg() {
        let buf = "WRITE_OFFSET 1234+2 42";
        let msg = RawInputMessage::from_str(&buf).unwrap();
        assert_eq!(
            msg,
            RawInputMessage::write_offset(
                Offset::from(0x1234, 2).unwrap(),
                Value::Number(42)));
    }

    #[test]
    fn should_parse_obs_lvar_msg() {
        let buf = "OBS_LVAR foobar";
        let msg = RawInputMessage::from_str(&buf).unwrap();
        assert_eq!(msg, RawInputMessage::obs_lvar("foobar"));
    }

    #[test]
    fn should_parse_obs_offset_msg() {
        let buf = "OBS_OFFSET 330+2";
        let msg = RawInputMessage::from_str(&buf).unwrap();
        assert_eq!(msg, RawInputMessage::obs_offset(Offset::from(0x0330, 2).unwrap()));
    }

    #[test]
    fn should_fail_to_parse_empty_line() {
        let buf = "";
        assert!(RawInputMessage::from_str(&buf).is_err());
    }
}

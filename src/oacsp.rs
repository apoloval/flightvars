//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;
use std::io;
use std::str::FromStr;

use byteorder::{BigEndian, ReadBytesExt};
use hex::FromHex;

#[derive(Debug, PartialEq)]
pub struct OffsetAddr(u16);

impl FromHex for OffsetAddr {
    type Error = io::Error;
    fn from_hex(s: &str) -> Result<Self, Self::Error> {
        let error = || io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid FSUIPC offset address in '{}'", s));
        let buf: Vec<u8> = try!(FromHex::from_hex(s).map_err(|_| error()));
        if buf.len() > 2 { Err(error()) }
        else {
            (&buf[..]).read_u16::<BigEndian>()
                .map(|v| OffsetAddr(v))
                .map_err(|_| error())
        }
    }
}

impl fmt::Display for OffsetAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:x}", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub enum OffsetLen { Ub, Sb, Uw, Sw, Ud, Sd }

impl fmt::Display for OffsetLen {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            OffsetLen::Ub => write!(f, "UB"),
            OffsetLen::Sb => write!(f, "SB"),
            OffsetLen::Uw => write!(f, "UW"),
            OffsetLen::Sw => write!(f, "SW"),
            OffsetLen::Ud => write!(f, "UD"),
            OffsetLen::Sd => write!(f, "SD"),
        }
    }
}

impl FromStr for OffsetLen {
    type Err = io::Error;
    fn from_str(s: &str) -> Result<OffsetLen, io::Error> {
        match s {
            "UB" => Ok(OffsetLen::Ub),
            "SB" => Ok(OffsetLen::Sb),
            "UW" => Ok(OffsetLen::Uw),
            "SW" => Ok(OffsetLen::Sw),
            "UD" => Ok(OffsetLen::Ud),
            "SD" => Ok(OffsetLen::Sd),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid FSUIPC offset length in '{}'", s))),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Offset(OffsetAddr, OffsetLen);

impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}:{}", self.0, self.1)
    }
}

impl FromStr for Offset {
    type Err = io::Error;
    fn from_str(s: &str) -> Result<Offset, io::Error> {
        let pair: Vec<&str> = s.split(":").collect();
        if pair.len() == 2 {
            let addr = try!(OffsetAddr::from_hex(pair[0]));
            let len = try!(OffsetLen::from_str(pair[1]));
            Ok(Offset(addr, len))
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid FSUIPC offset in '{}'", s)))
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Message {
    Begin { version: u16, client_id: String },
    WriteLvar { lvar: String, value: i64 },
    WriteOffset { offset: Offset, value: i64 },
    ObserveLvar { lvar: String }
}

impl Message {

    pub fn begin(version: u16, client_id: &str) -> Message {
        Message::Begin { version: version, client_id: client_id.to_string() }
    }

    pub fn write_lvar(lvar: &str, value: i64) -> Message {
        Message::WriteLvar { lvar: lvar.to_string(), value: value }
    }

    pub fn write_offset(offset: Offset, value: i64) -> Message {
        Message::WriteOffset { offset: offset, value: value }
    }

    pub fn obs_lvar(lvar: &str) -> Message {
        Message::ObserveLvar { lvar: lvar.to_string() }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &Message::Begin { version, ref client_id } =>
                write!(f, "BEGIN {} {}", version, client_id),
            &Message::WriteLvar { ref lvar, value} =>
                write!(f, "WRITE_LVAR {} {}", lvar, value),
            &Message::WriteOffset { ref offset, value } =>
                write!(f, "WRITE_OFFSET {} {}", offset, value),
            &Message::ObserveLvar { ref lvar } =>
                write!(f, "OBS_LVAR {}", lvar),
        }
    }
}

impl FromStr for Message {
    type Err = io::Error;
    fn from_str(s: &str) -> Result<Message, io::Error> {
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

    pub fn parse(self) -> io::Result<Message> {
        let args: Vec<&str> = self.input.split_whitespace().collect();
        let cmd = args[0];
        let args = &args[1..];
        match &cmd.to_uppercase()[..] {
            "BEGIN" => self.parse_begin(&args),
            "WRITE_LVAR" => self.parse_write_lvar(&args),
            "WRITE_OFFSET" => self.parse_write_offset(&args),
            "OBS_LVAR" => self.parse_obs_lvar(&args),
            _ => Err(self.input_error()),
        }
    }

    fn parse_begin(self, args: &[&str]) -> io::Result<Message> {
        try!(self.require_argc(args, 2));
        let ver = try!(args[0].parse().map_err(|_| self.input_error()));
        Ok(Message::begin(ver, args[1]))
    }

    fn parse_write_lvar(self, args: &[&str]) -> io::Result<Message> {
        try!(self.require_argc(args, 2));
        let lvar = args[0];
        let value = try!(args[1].parse().map_err(|_| self.input_error()));
        Ok(Message::write_lvar(&lvar, value))
    }

    fn parse_write_offset(self, args: &[&str]) -> io::Result<Message> {
        try!(self.require_argc(args, 2));
        let offset = try!(args[0].parse());
        let value = try!(args[1].parse().map_err(|_| self.input_error()));
        Ok(Message::write_offset(offset, value))
    }

    fn parse_obs_lvar(self, args: &[&str]) -> io::Result<Message> {
        try!(self.require_argc(args, 1));
        Ok(Message::obs_lvar(args[0]))
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

    use super::*;

    #[test]
    fn should_display_begin_msg() {
        let msg = Message::begin(1, "arduino");
        let buf = format!("{}", msg);
        assert_eq!(buf, "BEGIN 1 arduino")
    }

    #[test]
    fn should_parse_begin_msg() {
        let buf = "BEGIN 1 arduino";
        let msg = Message::from_str(&buf).unwrap();
        assert_eq!(msg, Message::begin(1, "arduino"));
    }

    #[test]
    fn should_display_write_lvar_msg() {
        let msg = Message::write_lvar("foobar", 42);
        let buf = format!("{}", msg);
        assert_eq!(buf, "WRITE_LVAR foobar 42")
    }

    #[test]
    fn should_parse_write_lvar_msg() {
        let buf = "WRITE_LVAR foobar 42";
        let msg = Message::from_str(&buf).unwrap();
        assert_eq!(msg, Message::write_lvar("foobar", 42));
    }

    #[test]
    fn should_display_write_offset_msg() {
        let msg = Message::write_offset(Offset(OffsetAddr(0x1234), OffsetLen::Uw), 42);
        let buf = format!("{}", msg);
        assert_eq!(buf, "WRITE_OFFSET 1234:UW 42")
    }

    #[test]
    fn should_parse_write_offset_msg() {
        let buf = "WRITE_OFFSET 1234:UW 42";
        let msg = Message::from_str(&buf).unwrap();
        assert_eq!(msg, Message::write_offset(Offset(OffsetAddr(0x1234), OffsetLen::Uw), 42));
    }

    #[test]
    fn should_display_obs_lvar_msg() {
        let msg = Message::obs_lvar("foobar");
        let buf = format!("{}", msg);
        assert_eq!(buf, "OBS_LVAR foobar")
    }

    #[test]
    fn should_parse_obs_lvar_msg() {
        let buf = "OBS_LVAR foobar";
        let msg = Message::from_str(&buf).unwrap();
        assert_eq!(msg, Message::obs_lvar("foobar"));
    }
}

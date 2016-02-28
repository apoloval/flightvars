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

    pub fn decode<R: io::BufRead>(r: &mut R) -> io::Result<Message> {
        let mut line = String::new();
        try!(r.read_line(&mut line));
        let args: Vec<&str> = line.split_whitespace().collect();
        let cmd = args[0];
        let args = &args[1..];
        match &cmd.to_uppercase()[..] {
            "BEGIN" => Message::decode_begin(&line, &args),
            "WRITE_LVAR" => Message::decode_write_lvar(&line, &args),
            "WRITE_OFFSET" => Message::decode_write_offset(&line, &args),
            "OBS_LVAR" => Message::decode_obs_lvar(&line, &args),
            _ => Err(Message::input_error(&line)),
        }
    }

    pub fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        match self {
            &Message::Begin { version, ref client_id } =>
                write!(w, "BEGIN {} {}\n", version, client_id),
            &Message::WriteLvar { ref lvar, value} =>
                write!(w, "WRITE_LVAR {} {}\n", lvar, value),
            &Message::WriteOffset { ref offset, value } =>
                write!(w, "WRITE_OFFSET {} {}\n", offset, value),
            &Message::ObserveLvar { ref lvar } =>
                write!(w, "OBS_LVAR {}\n", lvar),
        }
    }

    fn decode_begin(line: &str, args: &[&str]) -> io::Result<Message> {
        try!(Message::require_argc(args, 2, line));
        let ver = try!(args[0].parse().map_err(|_| Message::input_error(line)));
        Ok(Message::begin(ver, args[1]))
    }

    fn decode_write_lvar(line: &str, args: &[&str]) -> io::Result<Message> {
        try!(Message::require_argc(args, 2, line));
        let lvar = args[0];
        let value = try!(args[1].parse().map_err(|_| Message::input_error(line)));
        Ok(Message::write_lvar(&lvar, value))
    }

    fn decode_write_offset(line: &str, args: &[&str]) -> io::Result<Message> {
        try!(Message::require_argc(args, 2, line));
        let offset = try!(args[0].parse());
        let value = try!(args[1].parse().map_err(|_| Message::input_error(line)));
        Ok(Message::write_offset(offset, value))
    }

    fn decode_obs_lvar(line: &str, args: &[&str]) -> io::Result<Message> {
        try!(Message::require_argc(args, 1, line));
        Ok(Message::obs_lvar(args[0]))
    }

    fn require_argc(args: &[&str], expected: usize, line: &str) -> io::Result<()> {
        if args.len() == expected { Ok(()) }
        else { Err(Message::input_error(line)) }
    }

    fn input_error(line: &str) -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid oacsp message in '{}'", line))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_encode_begin_msg() {
        let mut buf = vec![];
        let msg = Message::begin(1, "arduino");
        msg.encode(&mut buf).unwrap();
        assert_eq!(buf, b"BEGIN 1 arduino\n")
    }

    #[test]
    fn should_decode_begin_msg() {
        let mut buf = &b"BEGIN 1 arduino\n"[..];
        let msg = Message::decode(&mut buf).unwrap();
        assert_eq!(msg, Message::begin(1, "arduino"));
    }

    #[test]
    fn should_encode_write_lvar_msg() {
        let mut buf = vec![];
        let msg = Message::write_lvar("foobar", 42);
        msg.encode(&mut buf).unwrap();
        assert_eq!(buf, b"WRITE_LVAR foobar 42\n")
    }

    #[test]
    fn should_decode_write_lvar_msg() {
        let mut buf = &b"WRITE_LVAR foobar 42\n"[..];
        let msg = Message::decode(&mut buf).unwrap();
        assert_eq!(msg, Message::write_lvar("foobar", 42));
    }

    #[test]
    fn should_encode_write_offset_msg() {
        let mut buf = vec![];
        let msg = Message::write_offset(Offset(OffsetAddr(0x1234), OffsetLen::Uw), 42);
        msg.encode(&mut buf).unwrap();
        assert_eq!(buf, b"WRITE_OFFSET 1234:UW 42\n")
    }

    #[test]
    fn should_decode_write_offset_msg() {
        let mut buf = &b"WRITE_OFFSET 1234:UW 42\n"[..];
        let msg = Message::decode(&mut buf).unwrap();
        assert_eq!(msg, Message::write_offset(Offset(OffsetAddr(0x1234), OffsetLen::Uw), 42));
    }

    #[test]
    fn should_encode_obs_lvar_msg() {
        let mut buf = vec![];
        let msg = Message::obs_lvar("foobar");
        msg.encode(&mut buf).unwrap();
        assert_eq!(buf, b"OBS_LVAR foobar\n")
    }

    #[test]
    fn should_decode_obs_lvar_msg() {
        let mut buf = &b"OBS_LVAR foobar\n"[..];
        let msg = Message::decode(&mut buf).unwrap();
        assert_eq!(msg, Message::obs_lvar("foobar"));
    }
}

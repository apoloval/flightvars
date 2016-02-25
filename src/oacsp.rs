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
pub enum OffsetLength { Ub, Sb, Uw, Sw, Ud, Sd }

impl fmt::Display for OffsetLength {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            OffsetLength::Ub => write!(f, "UB"),
            OffsetLength::Sb => write!(f, "SB"),
            OffsetLength::Uw => write!(f, "UW"),
            OffsetLength::Sw => write!(f, "SW"),
            OffsetLength::Ud => write!(f, "UD"),
            OffsetLength::Sd => write!(f, "SD"),
        }
    }
}

impl FromStr for OffsetLength {
    type Err = io::Error;
    fn from_str(s: &str) -> Result<OffsetLength, io::Error> {
        match s {
            "UB" => Ok(OffsetLength::Ub),
            "SB" => Ok(OffsetLength::Sb),
            "UW" => Ok(OffsetLength::Uw),
            "SW" => Ok(OffsetLength::Sw),
            "UD" => Ok(OffsetLength::Ud),
            "SD" => Ok(OffsetLength::Sd),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid FSUIPC offset length in '{}'", s))),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Message {
    Begin { version: u16, client_id: String },
    WriteLvar { lvar: String, value: i64 },
    WriteOffset { address: OffsetAddr, length: OffsetLength, value: i64 }
}

impl Message {

    pub fn begin(version: u16, client_id: &str) -> Message {
        Message::Begin { version: version, client_id: client_id.to_string() }
    }

    pub fn write_lvar(lvar: &str, value: i64) -> Message {
        Message::WriteLvar { lvar: lvar.to_string(), value: value }
    }

    pub fn write_offset(addr: OffsetAddr, len: OffsetLength, value: i64) -> Message {
        Message::WriteOffset { address: addr, length: len, value: value }
    }

    pub fn decode<R: io::BufRead>(r: &mut R) -> io::Result<Message> {
        let mut line = String::new();
        try!(r.read_line(&mut line));
        let mut split = line.split_whitespace();
        let cmd = try!(split.next().ok_or(Message::input_error(&line)));
        match &cmd.to_uppercase()[..] {
            "BEGIN" => Message::decode_begin(&line, &mut split),
            "WRITE_LVAR" => Message::decode_write_lvar(&line, &mut split),
            "WRITE_OFFSET" => Message::decode_write_offset(&line, &mut split),
            _ => Err(Message::input_error(&line)),
        }
    }

    pub fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        match self {
            &Message::Begin { version, ref client_id } =>
                write!(w, "BEGIN {} {}\n", version, client_id),
            &Message::WriteLvar { ref lvar, value} =>
                write!(w, "WRITE_LVAR {} {}\n", lvar, value),
            &Message::WriteOffset { ref address, ref length, value } =>
                write!(w, "WRITE_OFFSET {}:{} {}\n", address, length, value),
        }
    }

    fn decode_begin<'a, I: Iterator<Item=&'a str>>(
            line: &str, args: &mut I) -> io::Result<Message> {
        let version = try!(args.next().ok_or(Message::input_error(line)));
        let client_id = try!(args.next().ok_or(Message::input_error(line)));
        let version = try!(version.parse().map_err(|_| Message::input_error(line)));
        Ok(Message::begin(version, &client_id))
    }

    fn decode_write_lvar<'a, I: Iterator<Item=&'a str>>(
            line: &str, args: &mut I) -> io::Result<Message> {
        let lvar = try!(args.next().ok_or(Message::input_error(line)));
        let value = try!(args.next().ok_or(Message::input_error(line)));
        let value = try!(value.parse().map_err(|_| Message::input_error(line)));
        Ok(Message::write_lvar(&lvar, value))
    }

    fn decode_write_offset<'a, I: Iterator<Item=&'a str>>(
            line: &str, args: &mut I) -> io::Result<Message> {
        let error = || Message::input_error(line);
        let mut offset = try!(args.next().ok_or_else(&error)).split(":");
        let addr = try!(offset.next().ok_or_else(&error));
        let addr: OffsetAddr = try!(OffsetAddr::from_hex(&addr).map_err(|_| error()));
        let len = try!(offset.next().ok_or_else(&error));
        let len = try!(len.parse().map_err(|_| error()));
        let value = try!(args.next().ok_or_else(&error));
        let value = try!(value.parse().map_err(|_| error()));
        Ok(Message::write_offset(addr, len, value))
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
        let msg = Message::write_offset(OffsetAddr(0x1234), OffsetLength::Uw, 42);
        msg.encode(&mut buf).unwrap();
        assert_eq!(buf, b"WRITE_OFFSET 1234:UW 42\n")
    }

    #[test]
    fn should_decode_write_offset_msg() {
        let mut buf = &b"WRITE_OFFSET 1234:UW 42\n"[..];
        let msg = Message::decode(&mut buf).unwrap();
        assert_eq!(msg, Message::write_offset(OffsetAddr(0x1234), OffsetLength::Uw, 42));
    }
}

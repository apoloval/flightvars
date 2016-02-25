//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

#[derive(Debug, PartialEq)]
pub enum Message {
    WriteLvar { lvar: String, value: i64 }
}

impl Message {

    pub fn write_lvar(lvar: &str, value: i64) -> Message {
        Message::WriteLvar { lvar: lvar.to_string(), value: value }
    }

    pub fn decode<R: io::BufRead>(r: &mut R) -> io::Result<Message> {
        let mut line = String::new();
        try!(r.read_line(&mut line));
        let mut split = line.split_whitespace();
        let cmd = try!(split.next().ok_or(Message::input_error(&line)));
        match &cmd.to_uppercase()[..] {
            "WRITE_LVAR" => Message::decode_write_lvar(&line, &mut split),
            _ => Err(Message::input_error(&line)),
        }
    }

    pub fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        match self {
            &Message::WriteLvar { ref lvar, ref value} =>
                write!(w, "WRITE_LVAR {} {}\n", lvar, value),
        }
    }

    fn decode_write_lvar<'a, I: Iterator<Item=&'a str>>(
            line: &str, args: &mut I) -> io::Result<Message> {
        let lvar = try!(args.next().ok_or(Message::input_error(line)));
        let value = try!(args.next().ok_or(Message::input_error(line)));
        let value = try!(value.parse().map_err(|_| Message::input_error(line)));
        Ok(Message::write_lvar(&lvar, value))
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
}

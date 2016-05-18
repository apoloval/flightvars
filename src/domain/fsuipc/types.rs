//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;
use std::io;
use std::str;

use byteorder::{LittleEndian, ReadBytesExt};
use hex::FromHex;

use domain::types::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OffsetAddr(u16);

impl From<u16> for OffsetAddr {
    fn from(val: u16) -> OffsetAddr { OffsetAddr(val) }
}

impl From<OffsetAddr> for u16 {
    fn from(val: OffsetAddr) -> u16 { val.0 }
}

impl fmt::Display for OffsetAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:x}", self.0)
    }
}

impl FromHex for OffsetAddr {
    type Error = io::Error;
    fn from_hex(s: &str) -> io::Result<OffsetAddr> {
        match u16::from_str_radix(s, 16) {
            Ok(n) => Ok(OffsetAddr(n)),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid FSUIPC offset address in '{}'", s))),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OffsetLen {
    UnsignedByte,
    SignedByte,
    UnsignedWord,
    SignedWord,
    UnsignedDouble,
    SignedDouble
}

impl OffsetLen {
    pub fn decode_value(&self, buffer: &[u8; 4]) -> Value {
        match *self {
            OffsetLen::UnsignedByte =>
                Value::UnsignedInt(buffer[0] as usize),
            OffsetLen::SignedByte =>
                Value::Int(buffer[0] as isize),
            OffsetLen::UnsignedWord =>
                Value::UnsignedInt((&buffer[..]).read_u16::<LittleEndian>().unwrap() as usize),
            OffsetLen::SignedWord =>
                Value::Int((&buffer[..]).read_i16::<LittleEndian>().unwrap() as isize),
            OffsetLen::UnsignedDouble =>
                Value::UnsignedInt((&buffer[..]).read_u32::<LittleEndian>().unwrap() as usize),
            OffsetLen::SignedDouble =>
                Value::Int((&buffer[..]).read_i32::<LittleEndian>().unwrap() as isize),
        }
    }
}

impl From<OffsetLen> for usize {
    fn from(len: OffsetLen) -> usize {
        match len {
            OffsetLen::UnsignedByte => 1,
            OffsetLen::SignedByte => 1,
            OffsetLen::UnsignedWord => 2,
            OffsetLen::SignedWord => 2,
            OffsetLen::UnsignedDouble => 4,
            OffsetLen::SignedDouble => 4,
        }
    }
}

impl fmt::Display for OffsetLen {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            OffsetLen::UnsignedByte => write!(f, "UB"),
            OffsetLen::SignedByte => write!(f, "SB"),
            OffsetLen::UnsignedWord => write!(f, "UW"),
            OffsetLen::SignedWord => write!(f, "SW"),
            OffsetLen::UnsignedDouble => write!(f, "UD"),
            OffsetLen::SignedDouble => write!(f, "SD"),
        }
    }
}

impl str::FromStr for OffsetLen {
    type Err = io::Error;
    fn from_str(s: &str) -> Result<OffsetLen, io::Error> {
        match s {
            "UB" => Ok(OffsetLen::UnsignedByte),
            "SB" => Ok(OffsetLen::SignedByte),
            "UW" => Ok(OffsetLen::UnsignedWord),
            "SW" => Ok(OffsetLen::SignedWord),
            "UD" => Ok(OffsetLen::UnsignedDouble),
            "SD" => Ok(OffsetLen::SignedDouble),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid FSUIPC offset length in '{}'", s))),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Offset(OffsetAddr, OffsetLen);

impl Offset {
    pub fn new(addr: OffsetAddr, len: OffsetLen) -> Offset { Offset(addr, len) }
    pub fn addr(&self) -> OffsetAddr { self.0 }
    pub fn len(&self) -> OffsetLen { self.1 }

    pub fn parse_value(&self, s: &str) -> io::Result<Value> {
        match self.len() {
            OffsetLen::UnsignedByte => Value::parse_uint(s),
            OffsetLen::SignedByte => Value::parse_int(s),
            OffsetLen::UnsignedWord => Value::parse_uint(s),
            OffsetLen::SignedWord => Value::parse_int(s),
            OffsetLen::UnsignedDouble => Value::parse_uint(s),
            OffsetLen::SignedDouble => Value::parse_int(s),
        }
    }
}

impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}:{}", self.0, self.1)
    }
}

impl str::FromStr for Offset {
    type Err = io::Error;
    fn from_str(s: &str) -> Result<Offset, io::Error> {
        let pair: Vec<&str> = s.split(":").collect();
        if pair.len() == 2 {
            let addr = try!(OffsetAddr::from_hex(pair[0]));
            let len = try!(OffsetLen::from_str(pair[1]));
            Ok(Offset::new(addr, len))
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid FSUIPC offset in '{}'", s)))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::str::FromStr;

    use hex::FromHex;

    use domain::types::Value;
    use super::*;

    #[test]
    fn should_decode_unsigned_byte_offset_value() {
        assert_eq!(
            OffsetLen::UnsignedByte.decode_value(&[42, 0, 0, 0]),
            Value::UnsignedInt(42));
    }

    #[test]
    fn should_decode_signed_byte_offset_value() {
        assert_eq!(
            OffsetLen::SignedByte.decode_value(&[42, 0, 0, 0]),
            Value::Int(42));
    }

    #[test]
    fn should_decode_unsigned_word_offset_value() {
        assert_eq!(
            OffsetLen::UnsignedWord.decode_value(&[0x01, 0x02, 0, 0]),
            Value::UnsignedInt(0x0201));
    }

    #[test]
    fn should_decode_signed_word_offset_value() {
        assert_eq!(
            OffsetLen::SignedWord.decode_value(&[0x01, 0x02, 0, 0]),
            Value::Int(0x0201));
    }

    #[test]
    fn should_decode_unsigned_double_offset_value() {
        assert_eq!(
            OffsetLen::UnsignedDouble.decode_value(&[0x01, 0x02, 0x03, 0x04]),
            Value::UnsignedInt(0x04030201));
    }

    #[test]
    fn should_decode_signed_double_offset_value() {
        assert_eq!(
            OffsetLen::SignedDouble.decode_value(&[0x01, 0x02, 0x03, 0x04]),
            Value::Int(0x04030201));
    }

    #[test]
    fn should_display_offset_addr() {
        assert_eq!(format!("{}", OffsetAddr::from(0x1234)), "1234");
        assert_eq!(format!("{}", OffsetAddr::from(0xabcd)), "abcd");
    }

    #[test]
    fn should_get_offset_addr_from_hex() {
        assert_eq!(OffsetAddr::from_hex("1234").unwrap(), OffsetAddr::from(0x1234));
        assert_eq!(OffsetAddr::from_hex("abcd").unwrap(), OffsetAddr::from(0xabcd));
    }

    #[test]
    fn should_fail_get_offset_addr_from_invalid_hex() {
        assert_eq!(OffsetAddr::from_hex("").unwrap_err().kind(), io::ErrorKind::InvalidInput);
        assert_eq!(OffsetAddr::from_hex("foobar").unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn should_display_offset_len() {
        assert_eq!(format!("{}", OffsetLen::UnsignedByte), "UB");
        assert_eq!(format!("{}", OffsetLen::SignedByte), "SB");
        assert_eq!(format!("{}", OffsetLen::UnsignedWord), "UW");
        assert_eq!(format!("{}", OffsetLen::SignedWord), "SW");
        assert_eq!(format!("{}", OffsetLen::UnsignedDouble), "UD");
        assert_eq!(format!("{}", OffsetLen::SignedDouble), "SD");
    }

    #[test]
    fn should_get_offset_len_from_str() {
        assert_eq!(OffsetLen::from_str("UB").unwrap(), OffsetLen::UnsignedByte);
        assert_eq!(OffsetLen::from_str("SB").unwrap(), OffsetLen::SignedByte);
        assert_eq!(OffsetLen::from_str("UW").unwrap(), OffsetLen::UnsignedWord);
        assert_eq!(OffsetLen::from_str("SW").unwrap(), OffsetLen::SignedWord);
        assert_eq!(OffsetLen::from_str("UD").unwrap(), OffsetLen::UnsignedDouble);
        assert_eq!(OffsetLen::from_str("SD").unwrap(), OffsetLen::SignedDouble);
    }

    #[test]
    fn should_fail_get_offset_len_from_invalid_str() {
        assert_eq!(OffsetLen::from_str("").unwrap_err().kind(), io::ErrorKind::InvalidInput);
        assert_eq!(OffsetLen::from_str("foobar").unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn should_display_offset() {
        assert_eq!(
            format!("{}", Offset::new(OffsetAddr::from(0x1234), OffsetLen::UnsignedWord)),
            "1234:UW");
        assert_eq!(
            format!("{}", Offset::new(OffsetAddr::from(0xabcd), OffsetLen::SignedByte)),
            "abcd:SB");
    }

    #[test]
    fn should_get_offset_from_str() {
        assert_eq!(
            Offset::from_str("1234:UW").unwrap(),
            Offset::new(OffsetAddr::from(0x1234), OffsetLen::UnsignedWord));
        assert_eq!(
            Offset::from_str("abcd:SB").unwrap(),
            Offset::new(OffsetAddr::from(0xabcd), OffsetLen::SignedByte));
    }

    #[test]
    fn should_fail_get_offset_from_invalid_str() {
        assert_eq!(Offset::from_str("").unwrap_err().kind(), io::ErrorKind::InvalidInput);
        assert_eq!(Offset::from_str("foobar").unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }
}

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

/// An offset into a data vector.
///
/// Some domains uses 16-bits offsets to reference an specific item in a data vector.
/// That's the case of FSUIPC or IOCP. The `Offset` type serves to this purpose by
/// specifying a 16-bits offset and the number of bytes the data occupies from there. 
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Offset(u16, u8);

impl Offset {
    pub fn from(addr: u16, size: u8) -> Option<Offset> {
        match size {
            1 | 2 | 4 | 8 => Some(Offset(addr, size)),
            _ => None,
        }
    }
}

impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:x}+{}", self.0, self.1)
    }
}

impl str::FromStr for Offset {
    type Err = io::Error;
    fn from_str(s: &str) -> io::Result<Offset> {
        let pair: Vec<&str> = s.split("+").collect();
        if pair.len() == 2 {
            let addr = try!(u16::from_str_radix(pair[0], 16)
            	.map_err(|_| io::Error::new(
                    io::ErrorKind::InvalidInput, 
                    format!("invalid FSUIPC offset in '{}'", s))));
            let size = try!(u8::from_str(pair[1])
                .map_err(|_| io::Error::new(	
                    io::ErrorKind::InvalidInput, 
                    format!("invalid FSUIPC offset in '{}'", s))));
            Ok(Offset(addr, size))
        } else {
            Err(io::Error::new(
                    io::ErrorKind::InvalidInput, 
                    format!("invalid FSUIPC offset in '{}'", s)))
        }
    }
}

/// A domain-agnostic variable
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Var {
    /// A variable referenced by its name. 
    Named(String),
    /// A variable referenced by its offset and width.
    Offset(Offset),
}

impl Var {
	#[cfg(test)]
    pub fn named(n: &str) -> Var { Var::Named(n.to_string()) }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Number(isize),
}

macro_rules! define_from_value {
    ($t:ty) => (
        impl From<Value> for $t {
            fn from(v: Value) -> $t {
                match v {
                    Value::Bool(true) => 1 as $t,
                    Value::Bool(false) => 0 as $t,
                    Value::Number(i) => i as $t,
                }
            }
        }
    );
}

define_from_value!(u8);
define_from_value!(i8);
define_from_value!(u16);
define_from_value!(i16);
define_from_value!(u32);
define_from_value!(i32);
define_from_value!(f64);

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Value::Bool(v) => write!(f, "{}", v),
            Value::Number(v) => write!(f, "{}", v),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn should_display_offset_addr() {
        assert_eq!(format!("{}", Offset(0x1234, 1)), "1234+1");
        assert_eq!(format!("{}", Offset(0xabcd, 2)), "abcd+2");
    }

    #[test]
    fn should_get_offset_addr_from_str() {
        assert_eq!(Offset::from_str("1234+1").unwrap(), Offset(0x1234, 1));
        assert_eq!(Offset::from_str("abcd+2").unwrap(), Offset(0xabcd, 2));
    }

    #[test]
    fn should_fail_get_offset_addr_from_invalid_str() {
        assert_eq!(Offset::from_str("").unwrap_err().kind(), io::ErrorKind::InvalidInput);
        assert_eq!(Offset::from_str("foobar").unwrap_err().kind(), io::ErrorKind::InvalidInput);
        assert_eq!(Offset::from_str("1234").unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }
}

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
use std::sync::mpsc;

use domain::fsuipc::Offset;

#[derive(Clone)]
pub struct Client(String, EventSender);

impl Client {
    pub fn new(s: &str, sender: EventSender) -> Client { Client(s.to_string(), sender) }
    pub fn name(&self) -> &str { &self.0 }
    pub fn sender(&self) -> &EventSender { &self.1 }
}

impl PartialEq for Client {
    fn eq(&self, other: &Client) -> bool {
        self.0 == other.0
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Client({}, ...)", self.0)
    }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Var {
    LVar(String),
    FsuipcOffset(Offset),
}

impl Var {
    pub fn lvar(n: &str) -> Var { Var::LVar(n.to_string()) }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(isize),
    UnsignedInt(usize),
    Float(f32),
}

impl Value {
    pub fn parse_int(s: &str) -> io::Result<Value> {
        let parsed = try!(s.parse().map_err(|e| io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("cannot parse integer value from '{}'", s)
        )));
        Ok(Value::Int(parsed))
    }

    pub fn parse_uint(s: &str) -> io::Result<Value> {
        let parsed = try!(s.parse().map_err(|e| io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("cannot parse unsigned integer value from '{}'", s)
        )));
        Ok(Value::UnsignedInt(parsed))
    }
}

macro_rules! define_from_value {
    ($t:ty) => (
        impl From<Value> for $t {
            fn from(v: Value) -> $t {
                match v {
                    Value::Bool(true) => 1 as $t,
                    Value::Bool(false) => 0 as $t,
                    Value::Int(i) => i as $t,
                    Value::UnsignedInt(i) => i as $t,
                    Value::Float(f) => f as $t,
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
            Value::Int(v) => write!(f, "{}", v),
            Value::UnsignedInt(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Observe(Var, Client),
    Write(Var, Value),
    Close(Client),
}

impl Command {
    pub fn var(&self) -> Option<&Var> {
        match self {
            &Command::Observe(ref v, _) => Some(v),
            &Command::Write(ref v, _) => Some(v),
            _ => None,
        }
    }

    pub fn client(&self) -> Option<&Client> {
        match self {
            &Command::Observe(_, ref c) => Some(c),
            _ => None,
        }
    }
}

pub type CommandSender = mpsc::Sender<Command>;
pub type CommandReceiver = mpsc::Receiver<Command>;


#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Update(Var, Value),
    Close,
}

pub type EventSender = mpsc::Sender<Event>;
pub type EventReceiver = mpsc::Receiver<Event>;

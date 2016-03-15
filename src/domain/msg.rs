//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;
use std::sync::mpsc;

#[derive(Clone)]
pub struct Client(String, EventSender);

impl Client {
    pub fn new(s: &str, sender: EventSender) -> Client { Client(s.to_string(), sender) }
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


#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Domain {
    Custom(String)
}

impl Domain {
    pub fn custom(s: &str) -> Domain { Domain::Custom(s.to_string()) }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Var {
    Name(String),
    Offset(u16),
}

impl Var {
    pub fn name(n: &str) -> Var { Var::Name(n.to_string()) }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i32),
    Float(f32),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Observe(Domain, Var, Client),
    Write(Domain, Var, Value),
}

impl Command {
    pub fn domain(&self) -> &Domain {
        match self {
            &Command::Observe(ref d, _, _) => d,
            &Command::Write(ref d, _, _) => d,
        }
    }

    pub fn client(&self) -> Option<&Client> {
        match self {
            &Command::Observe(_, _, ref c) => Some(c),
            _ => None,
        }
    }
}

pub type CommandSender = mpsc::Sender<Command>;
pub type CommandReceiver = mpsc::Receiver<Command>;


#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Update(Domain, Var, Value),
    Close,
}

pub type EventSender = mpsc::Sender<Event>;
pub type EventReceiver = mpsc::Receiver<Event>;

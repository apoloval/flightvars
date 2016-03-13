//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::sync::mpsc;

pub mod oacsp;
pub mod dummy;

pub type Domain = String;

#[derive(Clone, Debug, PartialEq)]
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
pub enum Request<T> {
    Observe(Domain, Var, T),
    Write(Domain, Var, Value),
    Close,
}

impl<T> Request<T> {
    pub fn observe(domain: &str, var: Var, observer: T) -> Request<T> {
        Request::Observe(domain.to_string(), var, observer)
    }

    pub fn write(domain: &str, var: Var, val: Value) -> Request<T> {
        Request::Write(domain.to_string(), var, val)
    }

    pub fn observer(&self) -> Option<&T> {
        match self {
            &Request::Observe(_, _, ref observer) => Some(observer),
            _ => None,
        }
    }
}

pub type RawRequest = Request<()>;

impl RawRequest {
    pub fn with_observer(self, observer: &EventSender) -> DomainRequest {
        match self {
            Request::Observe(dom, var, _) => Request::Observe(dom, var, observer.clone()),
            Request::Write(dom, var, val) => Request::Write(dom, var, val),
            Request::Close => Request::Close,
        }
    }
}

pub type DomainRequest = Request<EventSender>;

impl DomainRequest {
    pub fn into_raw(self) -> RawRequest {
        match self {
            Request::Observe(dom, var, _) => Request::Observe(dom, var, ()),
            Request::Write(dom, var, val) => Request::Write(dom, var, val),
            Request::Close => Request::Close,
        }
    }
}

pub type RawRequestSender = mpsc::Sender<RawRequest>;
pub type RawRequestReceiver = mpsc::Receiver<RawRequest>;
pub type DomainRequestSender = mpsc::Sender<DomainRequest>;
pub type DomainRequestReceiver = mpsc::Receiver<DomainRequest>;


#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Update(Domain, Var, Value),
    Close,
}

impl Event {
    pub fn update(domain: &str, var: Var, val: Value) -> Event {
        Event::Update(domain.to_string(), var, val)
    }
}

pub type EventSender = mpsc::Sender<Event>;
pub type EventReceiver = mpsc::Receiver<Event>;


pub trait MessageRead {
    fn read_msg(&mut self) -> io::Result<RawRequest>;
}


pub trait MessageWrite {
    fn write_msg(&mut self, msg: &Event) -> io::Result<()>;
}


pub trait Protocol<I, O> {
    type Read: MessageRead;
    type Write: MessageWrite;
    fn reader(&self, input: I) -> Self::Read;
    fn writer(&self, output: O) -> Self::Write;
}

pub trait BidirProtocol<T> : Protocol<T, T> {}

impl<T, P: Protocol<T, T>> BidirProtocol<T> for P {}

pub fn oacsp() -> oacsp::Oacsp { oacsp::Oacsp }
pub fn dummy() -> dummy::DummyProtocol { dummy::DummyProtocol }

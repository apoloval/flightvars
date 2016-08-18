//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

use types::*;

mod fsuipc;
mod lvar;

pub struct Event {
    pub device: DeviceId,
    pub variable: Var,
    pub value: Value,
}

impl Event {
    pub fn new(device: DeviceId, variable: Var, value: Value) -> Event {
        Event {
            device: device,
            variable: variable,
            value: value,
        }
    }
}

pub trait Domain {
    fn write(&mut self, variable: &Var, value: &Value) -> io::Result<()>;
    fn subscribe(&mut self, device: DeviceId, variable: &Var) -> io::Result<()>;
    fn unsubscribe_all(&mut self, device: DeviceId) -> io::Result<()>;
    fn poll(&mut self, events: &mut Vec<Event>) -> io::Result<()>;
}

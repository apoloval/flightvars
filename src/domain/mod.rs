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

pub trait Domain {
    fn write(&mut self, variable: &Var, value: &Value) -> io::Result<()>;
    fn subscribe(&mut self, device: DeviceId, variable: &Var) -> io::Result<()>;
    fn unsubscribe_all(&mut self, device: DeviceId) -> io::Result<()>;
    fn poll<F: FnMut(DeviceId, Var, Value)>(&mut self, f: F) -> io::Result<()>;
}

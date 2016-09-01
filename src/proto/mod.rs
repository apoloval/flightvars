//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

mod oacsp;

pub use self::oacsp::Oacsp;

use io::DeviceHandler;
use types::{Value, Var};

pub trait Protocol : DeviceHandler {

	fn send_update(&mut self, domain: &str, variable: Var, value: Value) -> io::Result<()>;    
}

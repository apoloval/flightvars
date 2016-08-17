//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::result;

use types::*;

pub enum Error {}

pub type Result<T> = result::Result<T, Error>;

pub trait Domain<C> {
    fn write(variable: &Var, value: &Value) -> Result<()>;
    fn subscribe(client: C, variable: &Var) -> Result<()>;
    fn unsubscribe_all(client: C) -> Result<()>;
}

//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[macro_use]
extern crate log;

extern crate byteorder;
extern crate hex;
extern crate libc;
extern crate log4rs;

#[cfg(windows)]
mod fsx;

mod bus;
mod io;
mod oacsp;

#[cfg(windows)]
#[export_name="\x01_DLLStart"]
pub extern "stdcall" fn dll_start() {
    fsx::module::start_module();
}

#[cfg(windows)]
#[export_name="\x01_DLLStop"]
pub extern "stdcall" fn dll_stop() {
    fsx::module::stop_module();
}

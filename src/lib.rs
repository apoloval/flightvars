//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate byteorder;
extern crate fsuipc;
extern crate hex;
extern crate libc;
#[macro_use] extern crate log;
extern crate log4rs;
extern crate rustc_serialize;
extern crate serial;
extern crate tempdir;    
extern crate toml;
extern crate ws2_32;

// mod domain;
// #[cfg(windows)] mod fsx;
// mod comm;
mod io;
// mod port;
// mod proto;
// mod util;

#[cfg(windows)]
#[export_name="\x01_DLLStart"]
pub extern "stdcall" fn dll_start() {
    // fsx::module::start_module();
}

#[cfg(windows)]
#[export_name="\x01_DLLStop"]
pub extern "stdcall" fn dll_stop() {
    // fsx::module::stop_module();
}

// Making this public we ensure this symbol is exported in the DLL
// pub use domain::lvar::Panels;

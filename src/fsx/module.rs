//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::mem::size_of;
use std::ptr;
use std::sync::mpsc;

use libc::malloc;

use fsx::logging;
use port;
use proto;

struct Module {
    oacsp_tcp: Option<port::TcpPort>
}

impl Module {
    pub fn new() -> Self {
        Module {
            oacsp_tcp: None,
        }
    }

    pub fn start(&mut self) {
        logging::config_logging();
        info!("Starting FlightVars module v{}", FLIGHTVARS_VERSION);
        let (tx, _) = mpsc::channel();
        self.oacsp_tcp = Some(port::TcpPort::tcp_oacsp("0.0.0.0:1801", tx).unwrap());
        info!("FlightVars module started successfully");
    }
    pub fn stop(self) {
        info!("Stopping FlightVars module");
        for port in self.oacsp_tcp { port.shutdown(); }
        info!("FlightVars module stopped successfully");
    }
}

const FLIGHTVARS_VERSION: &'static str = "0.1.0";

static mut MODULE: *mut Module = 0 as *mut Module;

pub fn start_module() {
    unsafe {
        MODULE = malloc(size_of::<Module>()) as *mut Module;
        ptr::write(MODULE, Module::new());
        (*MODULE).start();
    }
}

pub fn stop_module() {
    unsafe {
        let m = ptr::read(MODULE);
        m.stop();
    }
}

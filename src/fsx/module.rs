//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::mem::size_of;
use std::ptr;

use libc::malloc;

use fv::*;
use config;
use fsx::logging;

const CONFIG_FILE: &'static str = "Modules/flightvars.conf";

struct Module {
    flightvars: Option<FlightVarsHandler>,
}

impl Module {
    pub fn new() -> Self {
        Module { flightvars: None }
    }

    pub fn start(&mut self) {
        let settings = config::Settings::from_toml_file(CONFIG_FILE)
        	.ok()
        	.unwrap_or_else(|| {
    	        println!("FlightVars cannot load config file at {}", CONFIG_FILE);
    	        println!("Falling back to default settings");
    	        config::Settings::default()
        	});
    	logging::config_logging(settings.logging);

        info!("Starting FlightVars module v{}", FLIGHTVARS_VERSION);
        self.flightvars = Some(FlightVars::new(&settings.oacsp_serial).unwrap());
        info!("FlightVars module started successfully");
    }
    pub fn stop(self) {
        info!("Stopping FlightVars module");
        for fv in self.flightvars {
            fv.close();
        }
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

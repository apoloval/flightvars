//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ffi::OsString;
use std::io;

use serial;

use comm;
use comm::Listen;
use domain::*;
use proto;
use port::Port;
use port::listen::*;
use util::Consume;


pub type SerialPort = Port<comm::serial::SerialInterruptor>;

impl SerialPort {
    pub fn new<D, P, S, I>(domain: D, proto: P, port_names: I) -> io::Result<SerialPort>
    where D: Consume<Item=Command> + Clone + Send + 'static,
          P: proto::Protocol<serial::SystemPort, serial::SystemPort> + Send + 'static,
          P::Read: Send + 'static,
          P::Write: Send + 'static,
          S: Into<OsString>,
          I: IntoIterator<Item=S> {
        let name = format!("serial/{} port", proto.name());
        info!("Creating {}", name);
        let mut scanner = try!(comm::serial::PortScanner::with_ports(port_names));
        let interruption = scanner.shutdown_interruption();
        Ok(Port {
            name: name,
            worker: ListenWorker::new(scanner, domain, proto, interruption),
        })
    }

    pub fn with_oacsp<D, S, I>(domain: D, port_names: I) -> io::Result<SerialPort>
    where D: Consume<Item=Command> + Clone + Send + 'static,
    	  S: Into<OsString>,
          I: IntoIterator<Item=S> {
        Self::new(domain, proto::oacsp(), port_names)
    }
}

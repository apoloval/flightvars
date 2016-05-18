//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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
    pub fn new<D, P>(domain: D, proto: P) -> io::Result<SerialPort>
    where D: Consume<Item=Command> + Clone + Send + 'static,
          P: proto::Protocol<serial::SystemPort, serial::SystemPort> + Send + 'static,
          P::Read: Send + 'static,
          P::Write: Send + 'static {
        let name = format!("serial/{} port", proto.name());
        info!("Creating {}", name);
        let mut scanner = try!(comm::serial::PortScanner::new());
        let interruption = scanner.shutdown_interruption();
        Ok(Port {
            name: name,
            worker: ListenWorker::new(scanner, domain, proto, interruption),
        })
    }

    pub fn with_oacsp<D>(domain: D) -> io::Result<SerialPort>
    where D: Consume<Item=Command> + Clone + Send + 'static {
        Self::new(domain, proto::oacsp())
    }
}

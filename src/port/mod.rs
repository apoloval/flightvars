//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;
use std::io;
use std::net;

use comm;
use comm::*;
use domain::*;
use proto;
use util::Consume;

mod conn;
mod listen;
mod read;
mod write;

use self::listen::*;

#[allow(dead_code)]
pub struct Port<I: comm::Interrupt> {
    name: String,
    worker: ListenWorker<I>
}

impl<I: comm::Interrupt> Port<I> {
    #[allow(dead_code)]
    pub fn shutdown(self) {
        info!("Shutting down {}", self.name);
        self.worker.shutdown();
    }
}


pub type TcpPort = Port<comm::tcp::TcpInterruptor>;

impl TcpPort {
    pub fn tcp<A, D, P>(name: String,
                        addr: A,
                        domain: D,
                        proto: P) -> io::Result<Port<comm::tcp::TcpInterruptor>>
    where A: net::ToSocketAddrs,
          D: Consume<Item=Command> + Clone + Send + 'static,
          P: proto::Protocol<net::TcpStream, net::TcpStream> + Send + 'static,
          P::Read: Send + 'static,
          P::Write: Send + 'static {
        info!("Creating {}", name);
        let mut listener = try!(comm::tcp::TcpListener::bind(addr));
        let interruption = listener.shutdown_interruption();
        Ok(Port {
            name: name,
            worker: ListenWorker::new(listener, domain, proto, interruption),
        })
    }

    pub fn tcp_oacsp<A, D>(addr: A, domain: D) ->
        io::Result<Port<comm::tcp::TcpInterruptor>>
    where A: net::ToSocketAddrs + fmt::Display,
          D: Consume<Item=Command> + Clone + Send + 'static {
        let name = format!("oacsp/tcp port at address {}", addr);
        Self::tcp(name, addr, domain, proto::oacsp())
    }
}

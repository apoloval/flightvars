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

mod worker;
mod spawn;

use self::worker::*;
use self::spawn::*;

#[allow(dead_code)]
pub struct Port<I: comm::Interrupt> {
    name: String,
    worker: Worker<I>
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
          P: proto::Protocol<comm::tcp::TcpInput, comm::tcp::TcpOutput> + Send + 'static,
          P::Read: Send + 'static,
          P::Write: Send + 'static {
        info!("Creating {}", name);
        let mut transport = try!(comm::tcp::TcpTransport::bind(addr));
        let interruption = transport.listener().shutdown_interruption();
        Ok(Port {
            name: name,
            worker: Worker::new(spawn_listener(transport, domain, proto), interruption),
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

#[cfg(test)]
pub type DummyPort = Port<comm::dummy::ListenerEventSender<proto::dummy::DummyCommand, Event>>;
#[cfg(test)]
pub type DummyPortListener = comm::dummy::ListenerEventSender<Command, Event>;
#[cfg(test)]
pub type DummyPortInput = comm::dummy::StreamEventSender<proto::dummy::DummyCommand>;
#[cfg(test)]
pub type DummyPortOutput = comm::dummy::MessageReceiver<Event>;

#[cfg(test)]
impl DummyPort {
    pub fn new<D>(domain: D) -> DummyPort
    where D: Consume<Item=Command> + Clone + Send + 'static {
        let listener = comm::dummy::DummyTransportListener::new();
        let mut transport = comm::dummy::DummyTransport::new(listener);
        let interruption = transport.listener().shutdown_interruption();
        let protocol = proto::dummy();
        let port = Port {
            name: "dummy".to_string(),
            worker: Worker::new(spawn_listener(transport, domain, protocol), interruption),
        };
        port
    }

    pub fn new_connection(&self) -> (DummyPortInput, DummyPortOutput) {
        self.worker.interruption().new_connection()
    }
}


#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use proto::dummy::DummyCommand;
    use domain::*;
    use super::*;

    #[test]
    fn should_open_and_close_port() {
        let (tx, _) = mpsc::channel();
        let port = DummyPort::new(tx);
        port.shutdown();
    }

    #[test]
    fn should_open_and_close_with_connections_established() {
        let (tx, _) = mpsc::channel();
        let port = DummyPort::new(tx);
        let (_, _) = port.new_connection();
        port.shutdown();
    }

    #[test]
    fn should_read_from_connection() {
        let (tx, rx) = mpsc::channel();
        let port = DummyPort::new(tx);
        let (conn_tx, _) = port.new_connection();
        let cmd = DummyCommand::Write(Var::lvar("var"), Value::Bool(true));
        conn_tx.send(cmd.clone());
        assert_eq!(DummyCommand::from(rx.recv().unwrap()), cmd);
        port.shutdown();
    }

    #[test]
    fn should_write_into_connection() {
        let (tx, rx) = mpsc::channel();
        let port = DummyPort::new(tx);
        let (conn_tx, conn_rx) = port.new_connection();
        let cmd = DummyCommand::Observe(Var::lvar("var"));
        conn_tx.send(cmd);
        let dom_cmd = rx.recv().unwrap();
        let client = dom_cmd.client().unwrap();
        let event = Event::Update(Var::lvar("var"), Value::Bool(true));
        client.sender().send(event.clone()).unwrap();
        assert_eq!(conn_rx.recv(), event);
        port.shutdown();
    }
}

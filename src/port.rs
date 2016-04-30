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
use std::sync::mpsc;
use std::thread;

use comm;
use comm::*;
use domain::*;
use proto;
use util::Consume;

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
            worker: Worker {
                thread: spawn_listener(transport, domain, proto),
                interruption: interruption
            }
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

pub type DummyPort = Port<comm::dummy::ListenerEventSender<proto::dummy::DummyCommand, Event>>;
pub type DummyPortListener = comm::dummy::ListenerEventSender<Command, Event>;
pub type DummyPortInput = comm::dummy::StreamEventSender<proto::dummy::DummyCommand>;
pub type DummyPortOutput = comm::dummy::MessageReceiver<Event>;

impl DummyPort {
    pub fn new<D>(domain: D) -> DummyPort
    where D: Consume<Item=Command> + Clone + Send + 'static {
        let listener = comm::dummy::DummyTransportListener::new();
        let mut transport = comm::dummy::DummyTransport::new(listener);
        let interruption = transport.listener().shutdown_interruption();
        let protocol = proto::dummy();
        let port = Port {
            name: "dummy".to_string(),
            worker: Worker {
                thread: spawn_listener(transport, domain, protocol),
                interruption: interruption
            }
        };
        port
    }

    pub fn new_connection(&self) -> (DummyPortInput, DummyPortOutput) {
        self.worker.interruption.new_connection()
    }
}

struct Worker<I> {
    thread: thread::JoinHandle<()>,
    interruption: I,
}

impl<I: comm::Interrupt> Worker<I> {
    pub fn shutdown(self) {
        self.interruption.interrupt();
        self.thread.join().unwrap();
    }
}

impl Worker<EventSender> {
    pub fn shutdown(self) {
        self.interruption.send(Event::Close).unwrap();
        self.thread.join().unwrap();
    }
}

struct Connection<I: comm::Interrupt> {
    reader: Worker<I>,
    writer: Worker<EventSender>
}

impl<I: comm::Interrupt> Connection<I> {
    pub fn shutdown(self) {
        info!("shutting down writer worker");
        self.writer.shutdown();
        info!("shutting down reader worker");
        self.reader.shutdown();
    }
}

fn spawn_listener<T, D, P>(mut transport: T,
                        domain: D,
                        proto: P) -> thread::JoinHandle<()>
where T: comm::Transport + Send + 'static,
      D: Consume<Item=Command> + Clone + Send + 'static,
      P: proto::Protocol<T::Input, T::Output> + Send + 'static,
      P::Read: Send + 'static,
      P::Write: Send + 'static {
    thread::spawn(move || {
        let mut connections = vec![];
        let listener = transport.listener();
        loop {
            match listener.listen() {
                Ok((input, output)) => {
                    info!("accepting a new connection from {}", input.id());
                    let conn = spawn_connection(input, output, domain.clone(), &proto);
                    connections.push(conn);
                },
                Err(ref e) if e.kind() == io::ErrorKind::ConnectionAborted => {
                    debug!("connection listener interrupted, closing");
                    break;
                },
                Err(e) => {
                    error!("unexpected error while listening for new connections: {}", e);
                    break;
                },
            }
        }
        for conn in connections {
            conn.shutdown();
        }
    })
}

fn spawn_connection<I, O, D, P>(input: I, output: O, domain: D, proto: &P) -> Connection<I::Int>
where I: comm::ShutdownInterruption + comm::Identify,
      D: Consume<Item=Command> + Send + 'static,
      P: proto::Protocol<I, O> + Send + 'static,
      P::Read: Send + 'static,
      P::Write: Send + 'static {
    let (reply_tx, reply_rx) = mpsc::channel();
    let client_name = input.id().to_string();
    let id = Client::new(&input.id(), reply_tx.clone());
    let mut reader_stream = input;
    let reader_interruption = reader_stream.shutdown_interruption();
    let msg_reader = proto.reader(reader_stream, id);
    let writer_stream = output;
    let writer_interruption = reply_tx;
    let msg_writer = proto.writer(writer_stream);
    let reader = spawn_reader(msg_reader, domain, client_name);
    let writer = spawn_writer(msg_writer, reply_rx);
    Connection {
        reader: Worker { thread: reader, interruption: reader_interruption },
        writer: Worker { thread: writer, interruption: writer_interruption }
    }
}

fn spawn_reader<R, D>(
	mut reader: R, 
	mut domain: D, 
	client_name: ClientName) -> thread::JoinHandle<()>
where R: proto::CommandRead + Send + 'static,
      D: Consume<Item=Command> + Send + 'static, {
    thread::spawn(move || {
        loop {
            match reader.read_cmd() {
                Ok(msg) => {
		            if let Err(_) = domain.consume(msg) {
		            	error!("unexpected error while consuming message");
		            }
                },
                Err(ref e) => {
                	if e.kind() == io::ErrorKind::ConnectionAborted {
                    	debug!("connection reset: terminating reader worker thread");
                	} else {
	                    error!("unexpected error ocurred, terminating reader thread: {}", e);
                	}
                    if let Err(_) = domain.consume(Command::Close(client_name)) {
                    	error!("unexpected error while consuming close message");
                    }
                    return;
                },
            };
        }
    })
}

fn spawn_writer<W>(mut writer: W,
                   output: EventReceiver) -> thread::JoinHandle<()>
where W: proto::EventWrite + Send + 'static {
    thread::spawn(move || {
        loop {
            let msg = output.recv().unwrap();
            if msg == Event::Close {
                debug!("terminating writer worker thread");
                return;
            }
            writer.write_ev(&msg).unwrap();
        }
    })
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

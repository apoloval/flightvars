//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::net;
use std::sync::mpsc;
use std::thread;

use comm;
use comm::*;
use proto;

#[allow(dead_code)]
pub struct Port<I: comm::Interrupt> {
    worker: Worker<I>
}

pub type TcpPort = Port<comm::tcp::TcpInterruptor>;

impl TcpPort {
    pub fn tcp<A, P>(addr: A,
                     domain_tx: mpsc::Sender<proto::MessageFrom>,
                     proto: P) -> io::Result<Port<comm::tcp::TcpInterruptor>>
    where A: net::ToSocketAddrs,
          P: proto::BidirProtocol<net::TcpStream> + Send + 'static,
          P::Read: Send + 'static,
          P::Write: Send + 'static {
        let mut transport = try!(comm::tcp::TcpTransport::bind(addr));
        let interruption = transport.listener().shutdown_interruption();
        Ok(Port { worker: Worker {
            thread: spawn_listener(transport, domain_tx, proto),
            interruption: interruption
        }})
    }

    pub fn tcp_oacsp<A>(addr: A, input: mpsc::Sender<proto::MessageFrom>) -> io::Result<Port<comm::tcp::TcpInterruptor>>
    where A: net::ToSocketAddrs {
        Self::tcp(addr, input, proto::oacsp())
    }
}

pub type DummyPort = Port<comm::dummy::ListenerEventSender<proto::RawMessage>>;
pub type DummyPortListener = comm::dummy::ListenerEventSender<proto::RawMessage>;
pub type DummyPortInput = comm::dummy::StreamEventSender<proto::RawMessage>;
pub type DummyPortOutput = comm::dummy::MessageReceiver<proto::RawMessage>;

impl DummyPort {
    pub fn new(domain_tx: mpsc::Sender<proto::MessageFrom>) -> DummyPort {
        let listener = comm::dummy::DummyTransportListener::new();
        let mut transport = comm::dummy::DummyTransport::new(listener);
        let interruption = transport.listener().shutdown_interruption();
        let protocol = proto::dummy();
        let port = Port { worker: Worker {
            thread: spawn_listener(transport, domain_tx, protocol),
            interruption: interruption
        }};
        port
    }

    pub fn new_connection(&self) -> (DummyPortInput, DummyPortOutput) {
        self.worker.interruption.new_connection()
    }

}

impl<I: comm::Interrupt> Port<I> {
    #[allow(dead_code)]
    pub fn shutdown(self) {
        self.worker.shutdown();
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

impl Worker<mpsc::Sender<proto::RawMessage>> {
    pub fn shutdown(self) {
        self.interruption.send(proto::Message::Close).unwrap();
        self.thread.join().unwrap();
    }
}

struct Connection<I: comm::Interrupt> {
    reader: Worker<I>,
    writer: Worker<mpsc::Sender<proto::RawMessage>>
}

impl<I: comm::Interrupt> Connection<I> {
    pub fn shutdown(self) {
        self.reader.shutdown();
        self.writer.shutdown();
    }
}

fn spawn_listener<T, P>(mut transport: T,
                        domain_tx: mpsc::Sender<proto::MessageFrom>,
                        proto: P) -> thread::JoinHandle<()>
where T: comm::Transport + Send + 'static,
      P: proto::Protocol<T::Input, T::Output> + Send + 'static,
      P::Read: Send + 'static,
      P::Write: Send + 'static {
    thread::spawn(move || {
        let mut connections = vec![];
        let listener = transport.listener();
        loop {
            match listener.listen() {
                Ok((input, output)) => {
                    let conn = spawn_connection(input, output, domain_tx.clone(), &proto);
                    connections.push(conn);
                },
                Err(_) => break,
            }
        }
        for conn in connections {
            conn.shutdown();
        }
    })
}

fn spawn_connection<I, O, P>(input: I,
                             output: O,
                             domain_tx: mpsc::Sender<proto::MessageFrom>,
                             proto: &P) -> Connection<I::Int>
where I: comm::ShutdownInterruption,
      P: proto::Protocol<I, O> + Send + 'static,
      P::Read: Send + 'static,
      P::Write: Send + 'static {
    let mut reader_stream = input;
    let reader_interruption = reader_stream.shutdown_interruption();
    let msg_reader = proto.reader(reader_stream);
    let (reply_tx, reply_rx) = mpsc::channel();
    let writer_stream = output;
    let writer_interruption = reply_tx.clone();
    let msg_writer = proto.writer(writer_stream);
    let reader = spawn_reader(msg_reader, domain_tx, reply_tx);
    let writer = spawn_writer(msg_writer, reply_rx);
    Connection {
        reader: Worker { thread: reader, interruption: reader_interruption },
        writer: Worker { thread: writer, interruption: writer_interruption }
    }
}

fn spawn_reader<R>(mut reader: R,
                   input: mpsc::Sender<proto::MessageFrom>,
                   output: mpsc::Sender<proto::RawMessage>) -> thread::JoinHandle<()>
where R: proto::MessageRead + Send + 'static {
    thread::spawn(move || {
        loop {
            let msg = match reader.read_msg() {
                Ok(msg) => msg,
                Err(ref e) if e.kind() == io::ErrorKind::ConnectionReset => {
                    info!("connection reset: terminating reader thread");
                    return;
                },
                Err(ref e) => {
                    error!("unexpected error ocurred, terminating reader thread: {}", e);
                    return;
                },
            };
            input.send(msg.map_origin(&output)).unwrap();
        }
    })
}

fn spawn_writer<W>(mut writer: W,
                   output: mpsc::Receiver<proto::RawMessage>) -> thread::JoinHandle<()>
where W: proto::MessageWrite + Send + 'static {
    thread::spawn(move || {
        loop {
            let msg = output.recv().unwrap();
            if msg == proto::Message::Close {
                return;
            }
            writer.write_msg(&msg).unwrap();
        }
    })
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use proto;
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
        conn_tx.send(proto::Message::Open);
        assert_eq!(rx.recv().unwrap().to_raw(), proto::Message::Open);
        port.shutdown();
    }

    #[test]
    fn should_write_into_connection() {
        let (tx, rx) = mpsc::channel();
        let port = DummyPort::new(tx);
        let (conn_tx, conn_rx) = port.new_connection();
        conn_tx.send(proto::Message::WriteData(42, ()));
        let req = rx.recv().unwrap();
        let origin = req.origin().unwrap();
        origin.send(proto::Message::Open).unwrap();
        assert_eq!(conn_rx.recv(), proto::Message::Open);
        port.shutdown();
    }
}

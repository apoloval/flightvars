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

use comm::Transport;
use proto;
use proto::ProtocolRead;

pub struct Port {
    thread: thread::JoinHandle<()>
}

impl Port {
    fn new<T, P>(trans: T, proto_read: P, chan: mpsc::Sender<proto::Message>) -> Port
    where T: Transport + Send + 'static,
          P: ProtocolRead<T::Input> + Send + 'static,
          P::IntoIter: Send {
        Port { thread: spawn_port_listener(trans, proto_read, chan) }
    }
}

fn spawn_port_listener<T, P>(
    mut trans: T, proto_read: P, chan: mpsc::Sender<proto::Message>) -> thread::JoinHandle<()>
where T: Transport + Send + 'static,
      P: ProtocolRead<T::Input> + Send + 'static,
      P::IntoIter: Send {
    thread::spawn(move || {
        loop {
            match trans.wait_conn() {
                Ok((r, _)) => {
                    let input = proto_read.iter_from(r);
                    let _reader = spawn_port_reader(input, chan.clone());
                },
                Err(_) => return,
            }
        }
    })
}

fn spawn_port_reader<I>(input: I, chan: mpsc::Sender<proto::Message>) -> thread::JoinHandle<()>
where I: IntoIterator<Item=proto::Message> + Send + 'static {
    thread::spawn(move|| {
        for msg in input.into_iter() {
            chan.send(msg).unwrap();
        }
    })
}

fn oacsp_tcp_port<A>(addr: A, incoming: mpsc::Sender<proto::Message>) -> io::Result<Port>
where A: net::ToSocketAddrs {
    let trans = try!(net::TcpListener::bind(addr));
    let proto = proto::oacsp();
    Ok(Port::new(trans, proto, incoming))
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::mpsc;

    use comm;
    use proto;

    use super::*;

    struct FakeTransport {
        connections: Vec<(mpsc::Receiver<proto::Message>, ())>
    }

    impl FakeTransport {
        fn new(input: mpsc::Receiver<proto::Message>) -> FakeTransport {
            FakeTransport {
                connections: vec![(input, ())]
            }
        }
    }

    struct FakeShutdownHandler;

    impl comm::ShutdownHandle for FakeShutdownHandler {
        fn shutdown(self) {}
    }

    impl comm::Transport for FakeTransport {
        type Input = mpsc::Receiver<proto::Message>;
        type Output = ();
        type Shutdown = FakeShutdownHandler;

        fn wait_conn(&mut self) -> io::Result<(mpsc::Receiver<proto::Message>, ())> {
            self.connections.pop().ok_or(io::Error::new(io::ErrorKind::ConnectionReset, ""))
        }

        fn shutdown_handle(&mut self) -> FakeShutdownHandler { FakeShutdownHandler }
    }

    #[test]
    fn should_receive_from_port() {
        let (net_tx, net_rx) = mpsc::channel();
        let (port_tx, port_rx) = mpsc::channel();
        net_tx.send(proto::Message::Open).unwrap();
        let transport = FakeTransport::new(net_rx);
        let _port = Port::new(transport, proto::id_proto(), port_tx);
        assert_eq!(proto::Message::Open, port_rx.recv().unwrap());
    }
}

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

use proto;

#[allow(dead_code)]
pub struct TcpPort {
    worker: TcpWorker<TcpShutdownHandler>
}

impl TcpPort {
    pub fn new<A, P>(addr: A,
                     input: mpsc::Sender<proto::MessageFrom>,
                     proto: P) -> io::Result<TcpPort>
    where A: net::ToSocketAddrs,
          P: proto::BidirProtocol<net::TcpStream> + Send + 'static,
          P::Read: Send + 'static,
          P::Write: Send + 'static {
        let listener = try!(net::TcpListener::bind(addr));
        let shutdown = TcpShutdownHandler::from_listener(&listener);
        Ok(TcpPort { worker: TcpWorker {
            thread: spawn_listener(listener, input, proto),
            shutdown: shutdown
        }})
    }

    #[allow(dead_code)]
    pub fn oacsp<A>(addr: A, input: mpsc::Sender<proto::MessageFrom>) -> io::Result<TcpPort>
    where A: net::ToSocketAddrs {
        Self::new(addr, input, proto::oacsp())
    }

    #[allow(dead_code)]
    pub fn shutdown(self) {
        self.worker.shutdown();
    }
}

struct TcpWorker<S> {
    thread: thread::JoinHandle<()>,
    shutdown: S,
}

impl TcpWorker<TcpShutdownHandler> {
    pub fn shutdown(self) {
        self.shutdown.shutdown();
        self.thread.join().unwrap();
    }
}

impl TcpWorker<mpsc::Sender<proto::RawMessage>> {
    pub fn shutdown(self) {
        self.shutdown.send(proto::Message::Close);
        self.thread.join().unwrap();
    }
}

struct TcpConnection {
    reader: TcpWorker<TcpShutdownHandler>,
    writer: TcpWorker<mpsc::Sender<proto::RawMessage>>
}

impl TcpConnection {
    pub fn shutdown(self) {
        self.reader.shutdown();
        self.writer.shutdown();
    }
}

fn spawn_listener<P>(listener: net::TcpListener,
                     input: mpsc::Sender<proto::MessageFrom>,
                     proto: P) -> thread::JoinHandle<()>
where P: proto::BidirProtocol<net::TcpStream> + Send + 'static,
      P::Read: Send + 'static,
      P::Write: Send + 'static {
    thread::spawn(move || {
        let mut connections = vec![];
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    let conn = spawn_connection(stream, input.clone(), &proto);
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

fn spawn_connection<P>(stream: net::TcpStream,
                       input: mpsc::Sender<proto::MessageFrom>,
                       proto: &P) -> TcpConnection
where P: proto::BidirProtocol<net::TcpStream> + Send + 'static,
      P::Read: Send + 'static,
      P::Write: Send + 'static {
    let reader_stream = stream.try_clone().unwrap();
    let reader_shutdown = TcpShutdownHandler::from_stream(&reader_stream);
    let msg_reader = proto.reader(reader_stream);
    let (output_tx, output_rx) = mpsc::channel();
    let writer_stream = stream.try_clone().unwrap();
    let writer_shutdown = output_tx.clone();
    let msg_writer = proto.writer(writer_stream);
    let reader = spawn_reader(msg_reader, input, output_tx);
    let writer = spawn_writer(msg_writer, output_rx);
    TcpConnection {
        reader: TcpWorker { thread: reader, shutdown: reader_shutdown },
        writer: TcpWorker { thread: writer, shutdown: writer_shutdown }
    }
}

fn spawn_reader<R>(mut reader: R,
                   input: mpsc::Sender<proto::MessageFrom>,
                   output: mpsc::Sender<proto::RawMessage>) -> thread::JoinHandle<()>
where R: proto::MessageRead + Send + 'static {
    thread::spawn(move || {
        loop {
            let msg = reader.read_msg().unwrap();
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

#[cfg(unix)]
pub mod unix {

    use std::net;
    use std::os::unix::io::{AsRawFd, RawFd};

    use libc;

    pub struct TcpShutdownHandler {
        fd: RawFd
    }

    impl TcpShutdownHandler {
        pub fn from_listener(listener: &net::TcpListener) -> TcpShutdownHandler {
            TcpShutdownHandler { fd: listener.as_raw_fd() }
        }

        pub fn from_stream(stream: &net::TcpStream) -> TcpShutdownHandler {
            TcpShutdownHandler { fd: stream.as_raw_fd() }
        }

        pub fn shutdown(self) {
            unsafe { libc::close(self.fd); }
        }
    }
}

pub use self::unix::*;

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn should_open_and_close_port() {
        let (tx, _) = mpsc::channel();
        let port = TcpPort::oacsp("127.0.0.1:2345", tx).unwrap();
        port.shutdown();
    }
}

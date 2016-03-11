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
    listener: thread::JoinHandle<()>
}

impl TcpPort {
    #[allow(dead_code)]
    pub fn new<A, P>(addr: A, input: mpsc::Sender<proto::MessageFrom>, proto: P) -> io::Result<TcpPort>
    where A: net::ToSocketAddrs,
          P: proto::Protocol<net::TcpStream> + Send + 'static,
          P::Decoder: Send + 'static {
        let listener = try!(net::TcpListener::bind(addr));
        Ok(TcpPort { listener: spawn_listener(listener, input, proto) })
    }
}

fn spawn_listener<P>(listener: net::TcpListener,
                     input: mpsc::Sender<proto::MessageFrom>,
                     proto: P) -> thread::JoinHandle<()>
where P: proto::Protocol<net::TcpStream> + Send + 'static,
      P::Decoder: Send + 'static {
    thread::spawn(move || {
        let mut connections = vec![];
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    let conn = spawn_connection(stream, input.clone(), &proto);
                    connections.push(conn);
                },
                Err(_) => return,
            }
        }
    })
}

struct TcpConnection {
    reader: thread::JoinHandle<()>,
    reader_shutdown: TcpShutdownHandler,
    writer: thread::JoinHandle<()>,
    writer_shutdown: TcpShutdownHandler
}

fn spawn_connection<P>(stream: net::TcpStream,
                       input: mpsc::Sender<proto::MessageFrom>,
                       proto: &P) -> TcpConnection
where P: proto::Protocol<net::TcpStream> + Send + 'static,
      P::Decoder: Send + 'static {
    let reader_stream = stream.try_clone().unwrap();
    let reader_shutdown = TcpShutdownHandler::from_stream(&reader_stream);
    let decoder = proto.decode(reader_stream);
    let writer_stream = stream.try_clone().unwrap();
    let writer_shutdown = TcpShutdownHandler::from_stream(&writer_stream);
    let (output_tx, output_rx) = mpsc::channel();
    let reader = spawn_reader(decoder, input, output_tx);
    let writer = spawn_writer(writer_stream, output_rx);
    TcpConnection {
        reader: reader,
        reader_shutdown: reader_shutdown,
        writer: writer,
        writer_shutdown: writer_shutdown
    }
}

fn spawn_reader<D>(decoder: D,
                   input: mpsc::Sender<proto::MessageFrom>,
                   output: mpsc::Sender<proto::RawMessage>) -> thread::JoinHandle<()>
where D: Iterator<Item=proto::RawMessage> + Send + 'static {
    thread::spawn(move || {
        for msg in decoder {
            input.send(msg.map_origin(&output)).unwrap();
        }
    })
}

fn spawn_writer(stream: net::TcpStream, output: mpsc::Receiver<proto::RawMessage>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {

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

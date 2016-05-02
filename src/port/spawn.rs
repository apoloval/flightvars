//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::sync::mpsc;
use std::thread;

use comm;
use comm::*;
use domain::*;
use port::worker::*;
use proto;
use util::Consume;


pub fn spawn_listener<T, D, P>(mut transport: T,
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
    Connection::new(
    	Worker::new(reader, reader_interruption),
        Worker::new(writer, writer_interruption))
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

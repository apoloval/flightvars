//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;
use std::io;
use std::sync::Arc;
use std::sync::atomic;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::thread;

use comm;
use comm::*;
use domain::*;
use port::worker::*;
use proto;
use util::Consume;


pub fn spawn_listener<L, D, P>(listener: L,
                        domain: D,
                        proto: P) -> thread::JoinHandle<()>
where L: comm::Listen + Send + 'static,
      D: Consume<Item=Command> + Clone + Send + 'static,
      P: proto::Protocol<L::Input, L::Output> + Send + 'static,
      P::Read: Send + 'static,
      P::Write: Send + 'static {
    thread::spawn(move || {
        let mut connections = vec![];
        let mut listener = listener;
        loop {
            match listener.listen() {
                Ok((input, output, addr)) => {
                    info!("accepting a new connection from {}", addr);
                    let conn = spawn_connection(input, output, addr, domain.clone(), &proto);
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

fn spawn_connection<I, O, A, D, P>(input: I, output: O, addr: A, domain: D, proto: &P) -> Connection
where A: fmt::Display,
      D: Consume<Item=Command> + Send + 'static,
      P: proto::Protocol<I, O> + Send + 'static,
      P::Read: Send + 'static,
      P::Write: Send + 'static {
    let reader_stop_signal = Arc::new(AtomicBool::new(false));
    let (writer_tx, writer_rx) = mpsc::channel();
    let client_name = addr.to_string();
    let id = Client::new(&client_name, writer_tx.clone());
    let msg_reader = proto.reader(input, id);
    let msg_writer = proto.writer(output);
    let reader = spawn_reader(msg_reader, domain, client_name, reader_stop_signal.clone());
    let writer = spawn_writer(msg_writer, writer_rx);
    Connection::new(
    	ReadWorker::new(reader, reader_stop_signal),
        WriteWorker::new(writer, writer_tx))
}

fn spawn_reader<R, D>(
	mut reader: R, 
	mut domain: D, 
	client_name: ClientName,
    stop_signal: Arc<AtomicBool>) -> thread::JoinHandle<()>
where R: proto::CommandRead + Send + 'static,
      D: Consume<Item=Command> + Send + 'static, {
    thread::spawn(move || {
        loop {
        	if stop_signal.load(atomic::Ordering::Relaxed) {
        		debug!("terminating due to stop signal detected");
        		return; 
        	}
            match reader.read_cmd() {
                Ok(msg) => {
		            if let Err(_) = domain.consume(msg) {
		            	error!("unexpected error while consuming message");
		            }
                },
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {},
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

//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::thread;

use comm;
use comm::*;
use domain::*;
use port::conn::*;
use proto;
use util::Consume;


pub struct ListenWorker<I: comm::Interrupt> {
    thread: thread::JoinHandle<()>,
    interruptor: I,
}

impl<I: comm::Interrupt> ListenWorker<I> {
    
	pub fn new<L, D, P>(listener: L,
                        domain: D,
                        proto: P, 
			   			interruptor: I) -> ListenWorker<I> 
	where L: comm::Listen + Send + 'static,
          D: Consume<Item=Command> + Clone + Send + 'static,
          P: proto::Protocol<L::Input, L::Output> + Send + 'static,
          P::Read: Send + 'static,
          P::Write: Send + 'static {
		let handle = spawn_listener(listener, domain, proto);			   			    
		ListenWorker {
			thread: handle,
			interruptor: interruptor,
		}
    }

    pub fn shutdown(self) {
        self.interruptor.interrupt();
        self.thread.join().unwrap();
    }
}

fn spawn_listener<L, D, P>(listener: L, domain: D, proto: P) -> thread::JoinHandle<()>
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
                    let conn = Connection::new(input, output, addr, domain.clone(), &proto);
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

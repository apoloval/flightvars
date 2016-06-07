//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::sync::Arc;
use std::sync::atomic;
use std::sync::atomic::AtomicBool;
use std::thread;

use domain::*;
use proto;
use util::Consume;


pub struct ReadWorker {
    thread: thread::JoinHandle<()>,
    signal: Arc<AtomicBool>,
}

impl ReadWorker {
    
    pub fn new<R, D>(reader: R, domain: D, client_name: ClientName) -> ReadWorker 
    where R: proto::CommandRead + Send + 'static,
          D: Consume<Item=Command> + Send + 'static {
	    let stop_signal = Arc::new(AtomicBool::new(false));
      	let handle = spawn_reader(reader, domain, client_name, stop_signal.clone());
        ReadWorker {
            thread: handle,
            signal: stop_signal,
        }
    }    

    pub fn shutdown(self) {
        self.signal.store(true, atomic::Ordering::Relaxed);
        self.thread.join().unwrap();
    }
}

fn spawn_reader<R, D>(reader: R, 
                      domain: D, 
                      client_name: ClientName,
                      stop_signal: Arc<AtomicBool>) -> thread::JoinHandle<()>
where R: proto::CommandRead + Send + 'static,
      D: Consume<Item=Command> + Send + 'static, {
    thread::spawn(move || {
        let mut reader = reader;
        let mut domain = domain;
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

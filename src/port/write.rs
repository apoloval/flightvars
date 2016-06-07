//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::sync::mpsc;
use std::thread;

use domain::*;
use proto;


pub struct WriteWorker {
    thread: thread::JoinHandle<()>,
    tx: EventSender,
}

impl WriteWorker {
	pub fn new<W>(writer: W) -> WriteWorker 
	where W: proto::EventWrite + Send + 'static {
	    let (tx, rx) = mpsc::channel();
	    let handle = spawn_writer(writer, rx);
		WriteWorker {
			thread: handle,
			tx: tx,
		}
	}
	
	pub fn channel(&self) -> EventSender {
	    self.tx.clone()
	}

    pub fn shutdown(self) {
        self.tx.send(Event::Close).unwrap();
        self.thread.join().unwrap();
    }
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

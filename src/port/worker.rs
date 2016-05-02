//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::thread;

use comm;
use comm::*;
use domain::*;

pub struct Worker<I: comm::Interrupt> {
    thread: thread::JoinHandle<()>,
    interruption: I,
}

impl<I: comm::Interrupt> Worker<I> {
	
	pub fn new(thread: thread::JoinHandle<()>, interruption: I) -> Worker<I> {
		Worker {
			thread: thread,
			interruption: interruption,
		}
	}
	
	pub fn interruption(&self) -> &I { &self.interruption }
	
    pub fn shutdown(self) {
        self.interruption.interrupt();
        self.thread.join().unwrap();
    }
}

pub struct Connection<I: comm::Interrupt> {
    reader: Worker<I>,
    writer: Worker<EventSender>
}

impl<I: comm::Interrupt> Connection<I> {
	
	pub fn new(reader: Worker<I>, writer: Worker<EventSender>) -> Connection<I> {
		Connection {
			reader: reader,
			writer: writer,
		}
	}
	
    pub fn shutdown(self) {
        info!("shutting down writer worker");
        self.writer.shutdown();
        info!("shutting down reader worker");
        self.reader.shutdown();
    }
}

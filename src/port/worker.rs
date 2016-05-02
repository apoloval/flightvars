//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::sync::Arc;
use std::sync::atomic;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::thread;

use comm;
use comm::*;
use domain::*;

pub trait Worker {
	fn shutdown(self);
}


pub struct ListenWorker<I: comm::Interrupt> {
    thread: thread::JoinHandle<()>,
    interruptor: I,
}

impl<I: comm::Interrupt> ListenWorker<I> {
	pub fn new(thread: thread::JoinHandle<()>, 
			   interruptor: I) -> ListenWorker<I> {
		ListenWorker {
			thread: thread,
			interruptor: interruptor,
		}
   }
}

impl<I: comm::Interrupt> Worker for ListenWorker<I> {		
    fn shutdown(self) {
        self.interruptor.interrupt();
        self.thread.join().unwrap();
    }
}


pub struct ReadWorker {
    thread: thread::JoinHandle<()>,
    signal: Arc<AtomicBool>,
}

impl ReadWorker {
	pub fn new(thread: thread::JoinHandle<()>, 
			   signal: Arc<AtomicBool>) -> ReadWorker {
		ReadWorker {
			thread: thread,
			signal: signal,
		}
	}	
}

impl Worker for ReadWorker {		
    fn shutdown(self) {
        self.signal.store(true, atomic::Ordering::Relaxed);
        self.thread.join().unwrap();
    }
}


pub struct WriteWorker {
    thread: thread::JoinHandle<()>,
    tx: mpsc::Sender<Event>,
}

impl WriteWorker {
	pub fn new(thread: thread::JoinHandle<()>, 
			   tx: mpsc::Sender<Event>) -> WriteWorker {
		WriteWorker {
			thread: thread,
			tx: tx,
		}
	}	
}

impl Worker for WriteWorker {		
    fn shutdown(self) {
        self.tx.send(Event::Close).unwrap();
        self.thread.join().unwrap();
    }
}


pub struct Connection {
    reader: ReadWorker,
    writer: WriteWorker,
}

impl Connection {
	
	pub fn new(reader: ReadWorker, writer: WriteWorker) -> Connection {
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

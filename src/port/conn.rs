//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;

use domain::*;
use port::read::*;
use port::write::*;
use proto;
use util::Consume;


pub struct Connection {
    reader: ReadWorker,
    writer: WriteWorker,
}

impl Connection {
	
	pub fn new<I, O, A, D, P>(input: I, output: O, addr: A, domain: D, proto: &P) -> Connection
    where A: fmt::Display,
          D: Consume<Item=Command> + Send + 'static,
          P: proto::Protocol<I, O> + Send + 'static,
          P::Read: Send + 'static,
          P::Write: Send + 'static {
        let writer = WriteWorker::new(proto.writer(output));
        let client_name = addr.to_string();
        let id = Client::new(&client_name, writer.channel());
        let reader = ReadWorker::new(proto.reader(input, id), domain, client_name);
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

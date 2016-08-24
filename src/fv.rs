//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::boxed::Box;
use std::io;
use std::sync::mpsc;
use std::thread;

use domain::DomainDispatcher;
use io::*;
use proto::*;

pub struct FlightVars {
    cmd_channel: mpsc::Receiver<FlightVarsCommand>,
    domains: DomainDispatcher,
    iocp: CompletionPort<Box<Protocol>>,
}

unsafe impl Send for FlightVars {}

impl FlightVars {
    
    pub fn new() -> io::Result<FlightVarsHandler> {
        let domains = try!(DomainDispatcher::new());
        let iocp = try!(CompletionPort::new());
        let (tx, rx) = mpsc::channel();
        let mut fv = FlightVars { cmd_channel: rx, domains: domains, iocp: iocp };
        let join_handle = thread::spawn(move || fv.run());
        let handler = FlightVarsHandler {
            join_handle: join_handle,
            cmd_channel: tx,
        };
        Ok(handler)
    }
    
    fn run(&mut self) {
        
    }
}

pub enum FlightVarsCommand {
    Close
}

pub struct FlightVarsHandler {
    join_handle: thread::JoinHandle<()>,
    cmd_channel: mpsc::Sender<FlightVarsCommand>   
}

impl FlightVarsHandler {
    
    pub fn close(self) {
        self.cmd_channel.send(FlightVarsCommand::Close).unwrap();
        self.join_handle.join().unwrap();
    }
}


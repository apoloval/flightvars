//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::thread;

use mio;

use domain::types::*;

pub mod types;
pub use self::types::*;

pub struct FsuipcDomain {
    worker: thread::JoinHandle<()>,
    tx: mio::Sender<Command>
}

impl FsuipcDomain {
    pub fn new() -> FsuipcDomain {
        let (worker, tx) = spawn_worker();
        FsuipcDomain { worker: worker, tx: tx }
    }

    pub fn shutdown() {
        // TODO: send a msg using tx and wait for worker to stop
        unimplemented!()
    }
}


pub struct FsuipcContext;

impl FsuipcContext {
    pub fn new()  -> FsuipcContext {
        FsuipcContext
    }
}

fn spawn_worker() -> (thread::JoinHandle<()>, mio::Sender<Command>) {
    let event_loop = mio::EventLoop::new().unwrap();
    let tx = event_loop.channel();
    let worker = thread::spawn(move || {
        let mut event_loop = event_loop;
        let mut ctx = FsuipcContext::new();
        event_loop.run(&mut ctx);
    });
    (worker, tx)
}

impl mio::Handler for FsuipcContext {
    type Timeout = ();
    type Message = Command;

    fn ready(&mut self,
             event_loop: &mut mio::EventLoop<FsuipcContext>,
             token: mio::Token,
             events: mio::EventSet) {
    }
}

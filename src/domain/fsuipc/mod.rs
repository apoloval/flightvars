//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::thread;

use fsuipc;
use fsuipc::{Handle, Session};
use mio;

use domain::types::*;
use util::Consume;

pub mod types;
pub use self::types::*;

pub enum Envelope {
    Cmd(Command),
    Shutdown
}

pub struct Domain {
    worker: thread::JoinHandle<()>,
    tx: mio::Sender<Envelope>
}

impl Domain {
    pub fn new() -> Domain {
        let (worker, tx) = spawn_worker();
        Domain { worker: worker, tx: tx }
    }

    pub fn shutdown(self) {
        self.tx.send(Envelope::Shutdown).unwrap();
        self.worker.join().unwrap();
    }

    pub fn consumer(&self) -> Consumer {
        Consumer { tx: self.tx.clone() }
    }
}

#[derive(Clone)]
pub struct Consumer {
    tx: mio::Sender<Envelope>
}

impl Consume for Consumer {
    type Item = Command;
    type Error = mio::NotifyError<Envelope>;
    fn consume(&mut self, cmd: Command) -> Result<(), mio::NotifyError<Envelope>> {
        self.tx.send(Envelope::Cmd(cmd))
    }
}


struct Context {
    handle: fsuipc::local::LocalHandle,
}

impl Context {
    pub fn new()  -> Context {
        Context {
            handle: fsuipc::local::LocalHandle::new().unwrap()
        }
    }

    fn process_write(&mut self, offset: Offset, value: Value) {
        debug!("writing value {} to offset {}", value, offset);
        match offset.len() {
            OffsetLen::UnsignedByte => self.process_write_of(offset.addr(), u8::from(value)),
            OffsetLen::SignedByte => self.process_write_of(offset.addr(), i8::from(value)),
            OffsetLen::UnsignedWord => self.process_write_of(offset.addr(), u16::from(value)),
            OffsetLen::SignedWord => self.process_write_of(offset.addr(), i16::from(value)),
            OffsetLen::UnsignedDouble => self.process_write_of(offset.addr(), u32::from(value)),
            OffsetLen::SignedDouble => self.process_write_of(offset.addr(), i32::from(value)),
        }
    }

    fn process_write_of<T>(&mut self, offset: OffsetAddr, value: T) {
        let mut session = self.handle.session();
        session.write(u16::from(offset), &value).unwrap();
        session.process().unwrap();
    }
}

fn spawn_worker() -> (thread::JoinHandle<()>, mio::Sender<Envelope>) {
    let event_loop = mio::EventLoop::new().unwrap();
    let tx = event_loop.channel();
    let worker = thread::spawn(move || {
        let mut event_loop = event_loop;
        let mut ctx = Context::new();
        event_loop.run(&mut ctx).unwrap();
    });
    (worker, tx)
}

impl mio::Handler for Context {
    type Timeout = ();
    type Message = Envelope;

    fn ready(&mut self,
             _event_loop: &mut mio::EventLoop<Context>,
             _token: mio::Token,
             _events: mio::EventSet) {
    }

    fn notify(&mut self, event_loop: &mut mio::EventLoop<Context>, msg: Envelope) {
        match msg {
            Envelope::Cmd(Command::Write(Var::FsuipcOffset(offset), value)) => {
                self.process_write(offset, value)
            },
            Envelope::Shutdown => event_loop.shutdown(),
            _ => {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_init_and_shutdown() {
        let mut domain = Domain::new();
        domain.shutdown();
    }
}

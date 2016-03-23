//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::thread;

use fsuipc;
use fsuipc::{Handle, Session};
use mio;

use domain::types::*;
use util::Consume;

pub mod types;
pub use self::types::*;

const POLLING_DELAY_MS: u64 = 50;

#[derive(Debug)]
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
        self.tx.send(Envelope::Shutdown).unwrap_or_else(|e| {
            warn!("unexpected error while sending shutdown message to FSUIPC domain: {}", e);
        });
        self.worker.join().unwrap_or_else(|_| {
            warn!("unexpected error while waiting for FSUIPC domain worker thread");
        });
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


struct Observer {
    offset: Offset,
    client: Client,
    retain: Option<Value>,
    buffer: [u8; 4],
}

impl Observer {
    pub fn read(&mut self, session: &mut fsuipc::local::LocalSession) {
        let offset = u16::from(self.offset.addr());
        let buffer = &mut self.buffer as *mut [u8; 4] as *mut u8;
        let nbytes = usize::from(self.offset.len());
        if let Err(e) = session.read_bytes(offset, buffer, nbytes) {
            error!("unexpected error while reading bytes from FSUIPC session: {}", e);
        }
    }

    pub fn trigger_event(&mut self) {
        let buf_value = self.offset.len().decode_value(&self.buffer);
        let must_trigger = self.retain.as_ref().map(|v| *v != buf_value).unwrap_or(true);
        if must_trigger {
            let event = Event::Update(Var::FsuipcOffset(self.offset), buf_value);
            debug!("triggering event {:?} to client {}", event, self.client.name());
            if let Err(e) = self.client.sender().send(event) {
                error!("expected error while sending event to client {}: {}",
                    self.client.name(), e);
            }
            self.retain = Some(buf_value);
        }
    }
}

struct Context {
    handle: fsuipc::local::LocalHandle,
    observers: Vec<Observer>,
}

impl Context {
    pub fn new()  -> io::Result<Context> {
        Ok(Context {
            handle: try!(fsuipc::local::LocalHandle::new()),
            observers: Vec::new(),
        })
    }

    fn process_write(&mut self, offset: Offset, value: Value) -> io::Result<()> {
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

    fn process_obs(&mut self, offset: Offset, client: Client) {
        debug!("client {} observing offset {}", client.name(), offset);
        self.observers.push(Observer {
            offset: offset,
            client: client,
            retain: None,
            buffer: [0; 4]
        });
    }

    fn process_write_of<T>(&mut self, offset: OffsetAddr, value: T) -> io::Result<()> {
        let mut session = self.handle.session();
        try!(session.write(u16::from(offset), &value));
        try!(session.process());
        Ok(())
    }
}

fn spawn_worker() -> (thread::JoinHandle<()>, mio::Sender<Envelope>) {
    let event_loop = mio::EventLoop::new().unwrap();
    let tx = event_loop.channel();
    let worker = thread::spawn(move || {
        let mut event_loop = event_loop;
        let mut ctx = match Context::new() {
            Ok(ctx) => ctx,
            Err(e) => {
                error!("cannot create FSUIPC context (is FSUIPC installed & running?): {}", e);
                return;
            },
        };
        event_loop.timeout_ms((), POLLING_DELAY_MS).unwrap();
        event_loop.run(&mut ctx).unwrap();
    });
    (worker, tx)
}

impl mio::Handler for Context {
    type Timeout = ();
    type Message = Envelope;

    fn timeout(&mut self, event_loop: &mut mio::EventLoop<Context>, _: ()) {
        let mut session = self.handle.session();
        for obs in self.observers.iter_mut() {
            obs.read(&mut session);
        }
        session.process().unwrap();
        for obs in self.observers.iter_mut() {
            obs.trigger_event();
        }
        event_loop.timeout_ms((), POLLING_DELAY_MS).unwrap();
    }

    fn notify(&mut self, event_loop: &mut mio::EventLoop<Context>, msg: Envelope) {
        match msg {
            Envelope::Cmd(Command::Write(Var::FsuipcOffset(offset), value)) => {
                if let Err(e) = self.process_write(offset, value) {
                    error!("unexpected IO error while processing write command: {}", e);
                }
            },
            Envelope::Cmd(Command::Observe(Var::FsuipcOffset(offset), client)) => {
                self.process_obs(offset, client)
            },
            Envelope::Shutdown => {
                info!("shutting down FSUIPC domain event loop");
                event_loop.shutdown();
            },
            other => {
                warn!("FSUIPC domain received an unexpected message: {:?}", other);
            },
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

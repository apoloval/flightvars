//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

use fsuipc;
use fsuipc::{Handle, Session};

use domain::types::*;
use domain::worker;
use util::Consume;

pub mod types;
pub use self::types::*;

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

pub struct Handler {
    handle: fsuipc::local::LocalHandle,
    observers: Vec<Observer>,
}

impl Handler {
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
    
    fn clean_obs(&mut self, client: ClientName) {
    	debug!("cleaning up observers for client {}", client);
    	self.observers.retain(|o| o.client.name() != client);
    }
}

impl worker::Handle for Handler {

    fn description() -> String { "fsuipc domain handler".to_string() }

    fn new() -> Handler {
        // TODO: do not unwrap here
        let handle = fsuipc::local::LocalHandle::new().unwrap();
        Handler {
            handle: handle,
            observers: Vec::new(),
        }
    }

    fn command(&mut self, cmd: Command) {
        match cmd {
            Command::Write(Var::FsuipcOffset(offset), value) => {
                if let Err(e) = self.process_write(offset, value) {
                    error!("unexpected IO error while processing write command: {}", e);
                }
            },
            Command::Observe(Var::FsuipcOffset(offset), client) => {
                self.process_obs(offset, client)
            },
            Command::Close(client) => {
            	self.clean_obs(client);
            },
			other =>
				warn!("FSUIPC domain received an unexpected command: {:?}", other),
    	}
    }

    fn poll(&mut self) {
        let mut session = self.handle.session();
        for obs in self.observers.iter_mut() {
            obs.read(&mut session);
        }
        if let Err(e) = session.process() {
            error!("unexpected error while processing FSUIPC session: {:?}", e);
            return;
        }
        for obs in self.observers.iter_mut() {
            obs.trigger_event();
        }
    }
}

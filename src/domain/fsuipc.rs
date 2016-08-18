//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

use byteorder::{BigEndian, ReadBytesExt};
use fsuipc::*;
use fsuipc::local::{LocalHandle, LocalSession};


use domain::*;
use types::*;

pub struct Fsuipc {
    handle: LocalHandle,
    subscriptions: Vec<Subscription>,
}

impl Fsuipc {
    pub fn new() -> io::Result<Fsuipc> {
        Ok(Fsuipc {
            handle: try!(LocalHandle::new()),
            subscriptions: Vec::new(),
        })
    }
    
    fn write_of<T>(&mut self, offset: u16, value: T) -> io::Result<()> {
        let mut session = self.handle.session();
        try!(session.write(u16::from(offset), &value));
        try!(session.process());
        Ok(())
    } 
}

impl Domain for Fsuipc {
    fn write(&mut self, variable: &Var, value: &Value) -> io::Result<()> {
        match variable {
            &Var::Offset(Offset(addr, 1)) => self.write_of::<u8>(addr, u8::from(value)),
            &Var::Offset(Offset(addr, 2)) => self.write_of::<u16>(addr, u16::from(value)),
            &Var::Offset(Offset(addr, 4)) => self.write_of::<u32>(addr, u32::from(value)),
            _ => unreachable!(),
        }
    }
    
    fn subscribe(&mut self, device: DeviceId, variable: &Var) -> io::Result<()> {
        match variable {
            &Var::Offset(ref offset) => {
                let subscription = Subscription {
                    device: device,
                    offset: offset.clone(),
                    retain: None,
                    buffer: [0; 4], 
                };
                self.subscriptions.push(subscription);
                Ok(())			
            },
            _ => {
                let error = io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("fsuipc domain cannot process subscription to variable {:?}", variable));
                Err(error)
            }
        }
    }
    
    fn unsubscribe_all(&mut self, device: DeviceId) -> io::Result<()> {
        self.subscriptions.retain(|s| s.device != device);
        Ok(())
    }
    
    fn poll(&mut self, events: &mut Vec<Event>) -> io::Result<()> {
        events.clear();
        let mut session = self.handle.session();
        for sub in self.subscriptions.iter_mut() {
            try!(sub.append_read(&mut session));
        }
        try!(session.process());
        for sub in self.subscriptions.iter_mut() {
            sub.trigger_event(events);
        }
        Ok(())
    }
}

struct Subscription {
    device: DeviceId,
    offset: Offset,
    retain: Option<[u8; 4]>,
    buffer: [u8; 4],
}

impl Subscription {
    fn append_read(&mut self, session: &mut LocalSession) -> io::Result<usize> {
        self.buffer = [0; 4];
        let offset = u16::from(self.offset.0);
        let buffer = &mut self.buffer as *mut [u8; 4] as *mut u8;
        let nbytes = usize::from(self.offset.1);
        session.read_bytes(offset, buffer, nbytes)
    }

    pub fn trigger_event(&mut self, events: &mut Vec<Event>) {
        let must_trigger = self.retain.as_ref().map(|v| *v != self.buffer).unwrap_or(true);
        if must_trigger {
            let value = match self.offset.1 {
                1 => Value::Number(self.buffer[0] as isize),
                2 => Value::Number((&self.buffer[0..1]).read_i16::<BigEndian>().unwrap() as isize),
                4 => Value::Number((&self.buffer[..]).read_i32::<BigEndian>().unwrap() as isize),
                _ => unreachable!(),  
            };
            self.retain = Some(self.buffer);
            let event = Event::new(self.device, Var::Offset(self.offset), value);
            events.push(event);
        }
    }
}

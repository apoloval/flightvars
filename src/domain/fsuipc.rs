//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::VecDeque;
use std::fmt::Display;
use std::io;

use byteorder::{BigEndian, ReadBytesExt};
use fsuipc::*;
use fsuipc::local::{LocalHandle, LocalSession};


use domain::*;
use types::*;

pub struct Fsuipc {
    handle: LocalHandle,
    subscriptions: Vec<Subscription>,
    writes: VecDeque<WriteOp>,
}

impl Fsuipc {
    pub fn new() -> io::Result<Fsuipc> {
        Ok(Fsuipc {
            handle: try!(LocalHandle::new()),
            subscriptions: Vec::new(),
            writes: VecDeque::with_capacity(1024),
        })
    }
    
    fn poll_writes(&mut self) -> io::Result<()> {
        loop {
            match self.writes.pop_front() {
                Some(op) => {
                    match self.poll_write(&op) {
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                            warn!("FSUIPC is not responding to write operations at this moment");
                            debug!("write operation {:?} is queued again to be processed later", 
                                op);
                            self.writes.push_front(op);
                            return Ok(());
                        }
                        Err(other) => return Err(other),
                        _ => {} 
                    }                    
                }
                None => return Ok(()),
            }
        }
    }
    
    fn poll_write(&mut self, write: &WriteOp) -> io::Result<()> {
        match *write {
            WriteOp::Byte(addr, byte) => self.write_of(addr, byte),
            WriteOp::Word(addr, word) => self.write_of(addr, word),
            WriteOp::DWord(addr, dword) => self.write_of(addr, dword),
        }
    }
    
    fn poll_subscriptions(&mut self, events: &mut Vec<Event>) -> io::Result<()> {
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

    fn write_of<T: Display>(&mut self, offset: u16, value: T) -> io::Result<()> {
        debug!("processing a write request for offset 0x{:x} <- {}", offset, value);
        let mut session = self.handle.session();
        try!(session.write(u16::from(offset), &value));
        try!(session.process());
        Ok(())
    } 
}

impl Domain for Fsuipc {
    fn write(&mut self, variable: &Var, value: &Value) -> io::Result<()> {
        debug!("queueing write operation for {:?} <- {}", variable, value);
        match variable {
            &Var::Offset(Offset(addr, 1)) => 
            	self.writes.push_back(WriteOp::Byte(addr, u8::from(value))), 
            &Var::Offset(Offset(addr, 2)) => 
            	self.writes.push_back(WriteOp::Word(addr, u16::from(value))),
            &Var::Offset(Offset(addr, 4)) => 
            	self.writes.push_back(WriteOp::DWord(addr, u32::from(value))),
            _ => unreachable!(),
        }
        Ok(())
    }
    
    fn subscribe(&mut self, device: DeviceId, variable: &Var) -> io::Result<()> {
        info!("receiving a subscription from device {} for {:?}", device, variable);
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
        debug!("removing all subscriptions for device ID {}", device);
        self.subscriptions.retain(|s| s.device != device);
        Ok(())
    }
    
    fn poll(&mut self, events: &mut Vec<Event>) -> io::Result<()> {
        try!(self.poll_writes());
        try!(self.poll_subscriptions(events));
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
            let event = Event::new(self.device, "fsuipc", Var::Offset(self.offset), value);
            events.push(event);
        }
    }
}

#[derive(Debug)]
enum WriteOp {
    Byte(u16, u8),
    Word(u16, u16),
    DWord(u16, u32),
}

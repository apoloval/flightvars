//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::rc::Rc;

use types::*;

mod fsuipc;
mod lvar;

pub struct Event {
    pub device: DeviceId,
    pub variable: Var,
    pub value: Value,
}

impl Event {
    pub fn new(device: DeviceId, variable: Var, value: Value) -> Event {
        Event {
            device: device,
            variable: variable,
            value: value,
        }
    }
}

pub trait Domain {
    fn write(&mut self, variable: &Var, value: &Value) -> io::Result<()>;
    fn subscribe(&mut self, device: DeviceId, variable: &Var) -> io::Result<()>;
    fn unsubscribe_all(&mut self, device: DeviceId) -> io::Result<()>;
    fn poll(&mut self, events: &mut Vec<Event>) -> io::Result<()>;
}

#[derive(Clone)]
pub struct DomainDispatcher {
    domains: HashMap<String, Rc<RefCell<Domain>>>,
}

impl DomainDispatcher {
    pub fn new() -> io::Result<DomainDispatcher> {
        let mut dispatcher = DomainDispatcher { domains: HashMap::new() };
        dispatcher.add("fsuipc", try!(fsuipc::Fsuipc::new()));
        dispatcher.add("lvar", lvar::LVar::new());
        Ok(dispatcher)
    }
    
    pub fn add<D: Domain + 'static>(&mut self, name: &str, d: D) {
        self.domains.insert(name.to_string(), Rc::new(RefCell::new(d)));
    }
    
    pub fn with_domain<F: FnOnce(&mut Domain)>(&mut self, name: &str, f: F) {
        if let Some(domain) = self.domains.get(name) {
        	f(&mut *domain.borrow_mut());
        }
    }
}

//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

mod ffi;

use std::ffi::CString;
use std::io;

use domain::*;
use types::*;

use self::ffi::*;

pub struct LVar {
    subscriptions: Vec<Subscription>,
}

impl LVar {
    pub fn new() -> LVar {
        LVar { subscriptions: Vec::new() }
    }
}

impl Domain for LVar {
    fn write(&mut self, variable: &Var, value: &Value) -> io::Result<()> {
        match variable {
            &Var::Named(ref lvar) => {
                let id = try!(check_named_variable(lvar).ok_or(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("there is no such named variable '{}'", lvar))));
                set_named_variable_value(id, f64::from(value));
                Ok(())
            }
            _ => {
                let error = io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("lvar domain does not support variable {:?}", variable));
                Err(error)
            }
        }
    }
    
    fn subscribe(&mut self, device: DeviceId, variable: &Var) -> io::Result<()> {
        match variable {
            &Var::Named(ref lvar) => {
                let subs = Subscription {
                    device: device,
                    lvar: lvar.clone(),
                    retain: None,
                };
                self.subscriptions.push(subs);
                Ok(())
            }
            _ => {
                let error = io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("lvar domain does not support variable {:?}", variable));
                Err(error)
            }
        }
    }
    
    fn unsubscribe_all(&mut self, device: DeviceId) -> io::Result<()> {
        self.subscriptions.retain(|s| s.device != device);
        Ok(())
    }
    
    fn poll<F: FnMut(DeviceId, Var, Value)>(&mut self, mut f: F) -> io::Result<()> {
        for sub in self.subscriptions.iter_mut() {
            sub.trigger_event(&mut f);
        }
        Ok(())
    }    
}

struct Subscription {
    device: DeviceId,
    lvar: String,
    retain: Option<Value>,
}

impl Subscription {
    fn trigger_event<F: FnMut(DeviceId, Var, Value)>(&mut self, f: &mut F) {
        let id = check_named_variable(&self.lvar).unwrap();
        let val = Value::Number(get_named_variable_value(id) as isize);
        let must_trigger = self.retain.as_ref().map(|v| *v != val).unwrap_or(true);
        if must_trigger {
            let var = Var::Named(self.lvar.clone());
            f(self.device, var, val);
        }
    }
}

fn check_named_variable(name: &str) -> Option<Id> {
    unsafe {
        let func = (*Panels).check_named_variable;
        let name = CString::new(name).unwrap();
        let id = (func)(name.as_ptr());
        if id != -1 { Some(id) } else { None }
    }
}

fn get_named_variable_value(id: Id) -> f64 {
    unsafe {
        let func = (*Panels).get_named_variable_value;
        (func)(id)
    }
}

fn set_named_variable_value(id: Id, value: f64) {
    unsafe {
        let func = (*Panels).set_named_variable_value;
        (func)(id, value)
    }
}

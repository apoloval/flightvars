//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

use domain::types::*;
use domain::worker;
use domain::lvar::panel::*;

pub struct Handler {
    observers: Vec<Observer>,
}

impl Handler {
    fn process_write(&mut self, lvar: &str, value: Value) -> io::Result<()> {
        debug!("writing value {} to lvar {}", value, lvar);
        let id = try!(check_named_variable(lvar).ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("there is no such named variable '{}'", lvar))));
        set_named_variable_value(id, f64::from(value));
        Ok(())
    }

    fn process_obs(&mut self, lvar: &str, client: Client) {
        debug!("client {} observing lvar {}", client.name(), lvar);
        self.observers.push(Observer {
            lvar: lvar.to_string(),
            client: client,
            retain: None,
        });
    }
    
    fn clean_obs(&mut self, client: ClientName) {
    	debug!("cleaning up observers for client {}", client);
    	self.observers.retain(|o| o.client.name() != client);
    }
}

impl worker::Handle for Handler {
    fn new() -> Handler {
        Handler {
            observers: Vec::new(),
        }
    }

    fn description() -> String { "lvar domain handler".to_string() }

    fn command(&mut self, cmd: Command) {
        match cmd {
            Command::Write(Var::LVar(lvar), value) => {
                if let Err(e) = self.process_write(&lvar, value) {
                    error!("unexpected IO error while writing LVAR {}: {}", lvar, e);
                }
            },
            Command::Observe(Var::LVar(lvar), client) => {
                self.process_obs(&lvar, client);
            },
            Command::Close(client) => {
            	self.clean_obs(client);
            },
            other => {
                warn!("LVAR domain received an unexpected command: {:?}", other);
            },
        }
    }

    fn poll(&mut self) {
        for obs in self.observers.iter_mut() {
            obs.trigger_event();
        }
    }
}

struct Observer {
    lvar: String,
    client: Client,
    retain: Option<Value>,
}

impl Observer {
    pub fn trigger_event(&mut self) {
        let id = check_named_variable(&self.lvar).unwrap();
        let val = Value::Int(get_named_variable_value(id) as isize);
        let must_trigger = self.retain.as_ref().map(|v| *v != val).unwrap_or(true);
        if must_trigger {
            let var = Var::LVar(self.lvar.clone());
            if let Err(e) = self.client.sender().send(Event::Update(var, val)) {
                error!("unexpected error while sending event to client {}: {}",
                    self.client.name(), e);
            }
            self.retain = Some(val);
        }
    }
}


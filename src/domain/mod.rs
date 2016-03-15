//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashMap;

pub mod msg;

pub use self::msg::*;

pub trait CommandDelivery {
    fn deliver(&mut self, cmd: Command);
}

impl CommandDelivery for CommandSender {
    fn deliver(&mut self, cmd: Command) {
        self.send(cmd).unwrap()
    }
}

pub struct DomainRouter {
    routes: HashMap<Domain, CommandSender>
}

impl DomainRouter {
    pub fn new() -> DomainRouter {
        DomainRouter { routes: HashMap::new() }
    }

    pub fn add_route(&mut self, domain: Domain, dest: CommandSender) {
        self.routes.insert(domain, dest);
    }
}

impl CommandDelivery for DomainRouter {
    fn deliver(&mut self, cmd: Command) {
        let route = self.routes.get_mut(cmd.domain());
        match route {
            Some(r) => r.deliver(cmd),
            None => error!("no route defined for domain {:?}", cmd.domain()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn should_deliver_using_router() {
        let mut router = DomainRouter::new();
        let (tx, rx) = mpsc::channel();
        router.add_route(Domain::custom("domain"), tx);
        let cmd = Command::Write(Domain::custom("domain"), Var::Offset(0x1234), Value::Bool(true));
        router.deliver(cmd.clone());
        assert_eq!(rx.recv().unwrap(), cmd);
    }
}

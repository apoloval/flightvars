//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub mod fsuipc;
pub mod types;

pub use self::types::*;

pub trait CommandDelivery {
    fn deliver(&mut self, cmd: Command);
}

impl CommandDelivery for CommandSender {
    fn deliver(&mut self, cmd: Command) {
        self.send(cmd).unwrap()
    }
}

pub struct DomainRouter {
    fsuipc: Option<CommandSender>,
    lvar: Option<CommandSender>,
}

impl DomainRouter {
    pub fn new() -> DomainRouter {
        DomainRouter {  fsuipc: None, lvar: None }
    }

    pub fn add_fsuipc_route(&mut self, dest: CommandSender) { self.fsuipc = Some(dest); }
    pub fn add_lvar_route(&mut self, dest: CommandSender) { self.lvar = Some(dest); }

    fn route_for(&mut self, var: &Var) -> Option<&mut CommandSender> {
        match var {
            &Var::LVar(_) => self.lvar.as_mut(),
            &Var::FsuipcOffset(_) => self.fsuipc.as_mut(),
        }
    }
}

impl CommandDelivery for DomainRouter {
    fn deliver(&mut self, cmd: Command) {
        let route = self.route_for(cmd.var());
        match route {
            Some(r) => r.deliver(cmd),
            None => error!("no route defined for variable {:?}", cmd.var()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use domain::fsuipc::types::*;

    use super::*;

    #[test]
    fn should_deliver_using_router() {
        let mut router = DomainRouter::new();
        let (tx, rx) = mpsc::channel();
        router.add_fsuipc_route(tx);
        let cmd = Command::Write(
            Var::FsuipcOffset(Offset::new(OffsetAddr::from(0x1234), OffsetLen::UnsignedWord)),
            Value::Bool(true));
        router.deliver(cmd.clone());
        assert_eq!(rx.recv().unwrap(), cmd);
    }
}

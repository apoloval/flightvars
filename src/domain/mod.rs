//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use util::Consume;

pub mod fsuipc;
pub mod lvar;
pub mod notify;
pub mod types;

pub use self::notify::*;
pub use self::types::*;

pub struct DomainRouter<F, L>
where F: Consume<Item=Command>,
      L: Consume<Item=Command> {
    fsuipc: F,
    lvar: L,
}

impl<F, L> DomainRouter<F, L>
where F: Consume<Item=Command>,
      L: Consume<Item=Command> {
    pub fn new(fsuipc: F, lvar: L) -> DomainRouter<F, L> {
        DomainRouter {  fsuipc: fsuipc, lvar: lvar }
    }
}

impl<F, L> Clone for DomainRouter<F, L>
where F: Consume<Item=Command> + Clone,
      L: Consume<Item=Command> + Clone {
    fn clone(&self) -> DomainRouter<F, L> {
        DomainRouter {  fsuipc: self.fsuipc.clone(), lvar: self.lvar.clone() }
    }
}

impl<F, L> Consume for DomainRouter<F, L>
where F: Consume<Item=Command>,
      L: Consume<Item=Command> {
    type Item = Command;
    type Error = ();
    fn consume(&mut self, cmd: Command) -> Result<(), ()> {
        match cmd.var() {
            &Var::LVar(_) => Ok(self.lvar.consume(cmd).unwrap_or(())),
            &Var::FsuipcOffset(_) => Ok(self.fsuipc.consume(cmd).unwrap_or(())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use domain::fsuipc::types::*;
    use util::{Consume, sink_consumer};

    use super::*;

    #[test]
    fn should_deliver_using_router() {
        let (tx, rx) = mpsc::channel();
        let mut router = DomainRouter::new(tx, sink_consumer());
        let cmd = Command::Write(
            Var::FsuipcOffset(Offset::new(OffsetAddr::from(0x1234), OffsetLen::UnsignedWord)),
            Value::Bool(true));
        router.consume(cmd.clone());
        assert_eq!(rx.recv().unwrap(), cmd);
    }
}

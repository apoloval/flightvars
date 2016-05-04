//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use comm;

mod conn;
mod listen;
mod read;
mod tcp;
mod write;

use self::listen::*;

pub use self::tcp::*;

#[allow(dead_code)]
pub struct Port<I: comm::Interrupt> {
    name: String,
    worker: ListenWorker<I>
}

impl<I: comm::Interrupt> Port<I> {
    #[allow(dead_code)]
    pub fn shutdown(self) {
        info!("Shutting down {}", self.name);
        self.worker.shutdown();
    }
}

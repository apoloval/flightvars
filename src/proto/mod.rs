//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

mod oacsp;

use types::{Value, Var};

pub enum InputMessage {
    Subscribe { domain: String, variable: Var },
    Write { domain: String, variable: Var, value: Value }
}

pub enum OutputMessage {
    Update { domain: String, variable: Var, value: Value }
}

pub trait Protocol {
    /// Decode a message from its serialized bytes.
    ///
    /// A common cause of failure is that no enough bytes were still received in
    /// that input. In that case, an error with a `io::ErrorKind::UnexpectedEof` 
    /// will be produced as result.
    fn decode<R: io::Read>(&mut self, input: &R) -> io::Result<InputMessage>;
    
    /// Encode a message into its serialized bytes.
    fn encode<W: io::Write>(&mut self, message: &OutputMessage, output: &W) -> io::Result<()>;
}

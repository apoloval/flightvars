//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;
use std::io;

use domain::types::*;
use domain::fsuipc::types::*;
use util::Consume;

#[derive(Clone, Debug, PartialEq)]
pub enum OutputMessage {
    EventLvar { lvar: String, value: Value },
    EventOffset { offset: OffsetAddr, value: Value }
}

impl OutputMessage {
	#[cfg(test)]
    pub fn event_lvar(lvar: &str, value: Value) -> OutputMessage {
        OutputMessage::EventLvar { lvar: lvar.to_string(), value: value }
    }

	#[cfg(test)]
    pub fn event_offset(offset: OffsetAddr, value: Value) -> OutputMessage {
        OutputMessage::EventOffset { offset: offset, value: value }
    }
}

impl fmt::Display for OutputMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &OutputMessage::EventLvar { ref lvar, value } =>
                write!(f, "EVENT_LVAR {} {}", lvar, value),
            &OutputMessage::EventOffset { ref offset, value } =>
                write!(f, "EVENT_OFFSET {} {}", offset, value),
        }
    }
}


pub struct MessageConsumer<W: io::Write> {
    output: W
}

impl<W: io::Write> MessageConsumer<W> {
    pub fn new(output: W) -> MessageConsumer<W> {
        MessageConsumer { output: output }
    }
}

impl<W: io::Write> Consume for MessageConsumer<W> {
    type Item = OutputMessage;
    type Error = io::Error;
    fn consume(&mut self, msg: OutputMessage) -> io::Result<()> {
        writeln!(&mut self.output, "{}", msg)
    }
}

#[cfg(test)]
mod tests {

    use domain::types::*;
    use domain::fsuipc::types::*;

    use super::*;

    #[test]
    fn should_display_event_lvar_msg() {
        let msg = OutputMessage::event_lvar("foobar", Value::Int(42));
        let buf = format!("{}", msg);
        assert_eq!(buf, "EVENT_LVAR foobar 42")
    }

    #[test]
    fn should_display_event_offset_msg() {
        let msg = OutputMessage::event_offset(OffsetAddr::from(0x1234), Value::UnsignedInt(42));
        let buf = format!("{}", msg);
        assert_eq!(buf, "EVENT_OFFSET 1234 42")
    }
}

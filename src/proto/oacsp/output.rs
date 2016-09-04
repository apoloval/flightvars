//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt;

use types::*;

#[derive(Clone, Debug, PartialEq)]
pub enum RawOutputMessage {
    EventLvar { lvar: String, value: Value },
    EventOffset { offset: Offset, value: Value }
}

impl RawOutputMessage {
	#[cfg(test)]
    pub fn event_lvar(lvar: &str, value: Value) -> RawOutputMessage {
        RawOutputMessage::EventLvar { lvar: lvar.to_string(), value: value }
    }

	#[cfg(test)]
    pub fn event_offset(offset: Offset, value: Value) -> RawOutputMessage {
        RawOutputMessage::EventOffset { offset: offset, value: value }
    }
}

impl fmt::Display for RawOutputMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &RawOutputMessage::EventLvar { ref lvar, value } =>
                write!(f, "EVENT_LVAR {} {}", lvar, value),
            &RawOutputMessage::EventOffset { ref offset, value } =>
                write!(f, "EVENT_OFFSET {} {}", offset, value),
        }
    }
}

#[cfg(test)]
mod tests {

    use types::*;

    use super::*;

    #[test]
    fn should_display_event_lvar_msg() {
        let msg = RawOutputMessage::event_lvar("foobar", Value::Number(42));
        let buf = format!("{}", msg);
        assert_eq!(buf, "EVENT_LVAR foobar 42")
    }

    #[test]
    fn should_display_event_offset_msg() {
        let msg = RawOutputMessage::event_offset(Offset::from(0x1234, 2).unwrap(), Value::Number(42));
        let buf = format!("{}", msg);
        assert_eq!(buf, "EVENT_OFFSET 1234+2 42")
    }
}

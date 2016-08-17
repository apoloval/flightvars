//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::io::BufRead;
use std::str::FromStr;

use proto::*;
use types::*;

mod input;
//mod output;
//mod reader;
//mod writer;

//use self::input::*;
//use self::output::*;
//pub use self::reader::*;
//pub use self::writer::*;

use self::input::RawInputMessage;

pub struct Oacsp {
    client_id: Option<String>
}

impl Oacsp {
    pub fn new() -> Oacsp {
        Oacsp { client_id: None }
    }
}

impl Protocol for Oacsp {
    
    fn decode<R: io::Read>(&mut self, input: R) -> io::Result<Decoded> {
        let mut buf = io::BufReader::new(input);
        let mut line = String::new();
        let nbytes = try!(buf.read_line(&mut line)) + 1; // end-of-line byte counts
        let begin_received = self.client_id.is_some();
        match (try!(RawInputMessage::from_str(&line)), begin_received) {
            (RawInputMessage::Begin { version: _, client_id }, false) => {
            	self.client_id = Some(client_id);
            	Ok(Decoded::ControlMessage(nbytes))
            },
            (RawInputMessage::Begin { version: _, client_id: _ }, true) => {
				Err(io::Error::new(io::ErrorKind::InvalidData, "begin message already received"))                    
            }
            (RawInputMessage::WriteLvar { lvar, value }, true) => {
                let msg = InputMessage::write("lvar", Var::Named(lvar), value);                
                Ok(Decoded::InputMessage(nbytes, msg))
            }
            (RawInputMessage::WriteOffset { offset, value }, true) => {
                let msg = InputMessage::write("fsuipc", Var::Offset(offset), value);                
                Ok(Decoded::InputMessage(nbytes, msg))
            }
            (RawInputMessage::ObserveLvar { lvar }, true) => {
                let msg = InputMessage::subscribe("lvar", Var::Named(lvar));                
                Ok(Decoded::InputMessage(nbytes, msg))
            }
            (RawInputMessage::ObserveOffset { offset }, true) => {
                let msg = InputMessage::subscribe("fsuipc", Var::Offset(offset));                
                Ok(Decoded::InputMessage(nbytes, msg))
            }
            (_, false) =>  {
				Err(io::Error::new(
				        io::ErrorKind::InvalidData, 
				        "unexpected message while waiting for begin"))                    
            }
        }                
    }
    
    fn encode<W: io::Write>(&mut self, _message: &OutputMessage, _output: &W) -> io::Result<()> {
        unimplemented!()
    }
    
}

#[cfg(test)]
mod test {
    
    use std::io;
    
    use proto::*;
    use types::*;
    
    use super::*;
    
    #[test]
    fn should_decode_begin() {
        let mut oacsp = Oacsp::new();
        let decoded = oacsp.decode(b"BEGIN 1 foobar" as &[u8]).unwrap();
        assert_eq!(decoded, Decoded::ControlMessage(15));
    }
    
    #[test]
    fn should_fail_decode_repeated_begin() {
        let mut oacsp = Oacsp::new();
        oacsp.decode(b"BEGIN 1 foobar" as &[u8]).unwrap();
        let error = oacsp.decode(b"BEGIN 1 foobar" as &[u8]).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }
        
    #[test]
    fn should_decode_write_lvar() {
        let mut oacsp = Oacsp::new();
        oacsp.decode(b"BEGIN 1 foobar" as &[u8]).unwrap();
        let decoded = oacsp.decode(b"WRITE_LVAR foobar 42" as &[u8]).unwrap();
        let expected = InputMessage::write("lvar", Var::named("foobar"), Value::Number(42));
        assert_eq!(decoded, Decoded::InputMessage(21, expected));
    }
        
    #[test]
    fn should_fail_decode_write_lvar_before_begin() {
        let mut oacsp = Oacsp::new();
        let error = oacsp.decode(b"WRITE_LVAR foobar 42" as &[u8]).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }
        
    #[test]
    fn should_decode_write_offset() {
        let mut oacsp = Oacsp::new();
        oacsp.decode(b"BEGIN 1 foobar" as &[u8]).unwrap();
        let decoded = oacsp.decode(b"WRITE_OFFSET 1234+2 42" as &[u8]).unwrap();
        let expected = InputMessage::write("fsuipc", Var::offset(0x1234, 2).unwrap(), Value::Number(42));
        assert_eq!(decoded, Decoded::InputMessage(23, expected));
    }
        
    #[test]
    fn should_fail_decode_write_offset_before_begin() {
        let mut oacsp = Oacsp::new();
        let error = oacsp.decode(b"WRITE_OFFSET 1234+2 42" as &[u8]).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }    
        
    #[test]
    fn should_decode_observe_lvar() {
        let mut oacsp = Oacsp::new();
        oacsp.decode(b"BEGIN 1 foobar" as &[u8]).unwrap();
        let decoded = oacsp.decode(b"OBS_LVAR foobar" as &[u8]).unwrap();
        let expected = InputMessage::subscribe("lvar", Var::named("foobar"));
        assert_eq!(decoded, Decoded::InputMessage(16, expected));
    }
        
    #[test]
    fn should_fail_decode_observe_lvar_before_begin() {
        let mut oacsp = Oacsp::new();
        let error = oacsp.decode(b"OBS_LVAR foobar" as &[u8]).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }    
        
    #[test]
    fn should_decode_observe_offset() {
        let mut oacsp = Oacsp::new();
        oacsp.decode(b"BEGIN 1 foobar" as &[u8]).unwrap();
        let decoded = oacsp.decode(b"OBS_OFFSET 1234+2" as &[u8]).unwrap();
        let expected = InputMessage::subscribe("fsuipc", Var::offset(0x1234, 2).unwrap());
        assert_eq!(decoded, Decoded::InputMessage(18, expected));
    }
        
    #[test]
    fn should_fail_decode_observe_offset_before_begin() {
        let mut oacsp = Oacsp::new();
        let error = oacsp.decode(b"OBS_OFFSET 1234+2" as &[u8]).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }    
}
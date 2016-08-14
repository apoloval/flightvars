//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ffi::OsStr;
use std::io;
use std::ops::{Deref, DerefMut};
use std::os::windows::ffi::OsStrExt;

use super::device::*;
use super::ffi::*;

pub struct Serial {
    dev: Device
}

impl Serial {
    
    pub fn open(port: &str) -> io::Result<Serial> {
        let encoded_port: Vec<u16> = OsStr::new(port)
        	.encode_wide()
        	.chain(Some(0).into_iter())
        	.collect();
        let handle = checked_handle! { valid =>
            CreateFileW(
                encoded_port.as_ptr() as LPCWSTR,
          		GENERIC_READ | GENERIC_WRITE,
          		0,
          		0 as LPSECURITY_ATTRIBUTES,
           		OPEN_EXISTING,
          		FILE_FLAG_OVERLAPPED,
          		0 as HANDLE)
        };

		// Set the appropriate timeouts for async IO
		let timeouts = COMMTIMEOUTS::for_async();
		checked_result!(SetCommTimeouts(handle, &timeouts as LPCCOMMTIMEOUTS));
		
        Ok(Serial {
            dev: Device::from_handle(handle)
        })
    }
    
    pub fn open_arduino(port: &str, baud_rate: usize) ->io::Result<Serial> {
        let mut port = try!(Serial::open(port));
        
        // Set the serial settings for Arduino
    	let mut dcb = try!(port.dcb());
		dcb.BaudRate = baud_rate as DWORD;
		dcb.ByteSize = 8;
		dcb.StopBits = 1;
		dcb.Parity = 0;
		dcb.setDtrControl();
		try!(port.set_dcb(&dcb));
		
		// Purge the buffers to eliminate accumulated messages prior to reset
        checked_result!(PurgeComm(port.handle(), PURGE_TXCLEAR | PURGE_RXCLEAR));
		Ok(port)
    }
    
    pub fn dcb(&self) -> io::Result<DCB> {
        let mut dcb = DCB::new();
        checked_result!(GetCommState(self.dev.handle(), &mut dcb as LPDCB));
        Ok(dcb)
    }
    
    pub fn set_dcb(&mut self, dcb: &DCB) -> io::Result<()> {
        checked_result!(SetCommState(self.dev.handle(), dcb as LPCDCB));
        Ok(())
    }
}

impl Deref for Serial {
    type Target = Device;
    
    fn deref(&self) -> &Device {
        &self.dev
    }
}

impl DerefMut for Serial {
    fn deref_mut(&mut self) -> &mut Device {
        &mut self.dev
    }
}

#[cfg(test)]
mod test {

	use std::time::Duration;

	use io::device::*;
	use io::iocp::*;

	use super::*;
	
	#[test]
	fn should_read() {
	    let mut iocp = CompletionPort::new().unwrap();	    
		let mut port = Serial::open_arduino("COM3", 9600).unwrap();
		iocp.attach(&port).unwrap();
		let mut pending_bytes = 6;
		while pending_bytes > 0 {
			port.request_read().unwrap();
    		let dev = iocp.process_event(&Duration::from_millis(5000)).unwrap();
    		assert_eq!(dev, port.id());
    		if let Event::BytesRead(bytes_read) = port.process_event() {
    		    pending_bytes -= bytes_read;
    		}
		}
		assert_eq!(port.recv_bytes(), b"Hello\n");
	}
}
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

#[allow(dead_code)]
pub enum SerialTimeouts {
    /// Read any byte upon available in input buffers
    ReadUponAvailable,
    
    /// Wait for all requested bytes to be available
    WaitToFill,
}

impl SerialTimeouts {
    fn as_raw(&self) -> COMMTIMEOUTS {
        match *self {
            SerialTimeouts::ReadUponAvailable => COMMTIMEOUTS::read_upon_available(),
            SerialTimeouts::WaitToFill => COMMTIMEOUTS::wait_to_fill(),
        }
    }
}

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
    
    pub fn set_timeouts(&mut self, timeouts: &SerialTimeouts) -> io::Result<()> {
        let raw = timeouts.as_raw();
		checked_result!(SetCommTimeouts(self.dev.handle(), &raw as LPCCOMMTIMEOUTS));
		Ok(())
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
    
    pub fn as_device(self) -> Device {
        self.dev
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

	use std::io;
	use std::{thread, time};

	use io::device::*;
	use io::iocp::*;

	use super::*;
	
	/// A test that uses an Arduino board connected to serial port to interact with the machine.
    /// The following Arduino sketch is suggested:
    ///
    /// ```c++
    /// void setup() {
    ///   Serial.begin(9600);
    ///   Serial.setTimeout(50);
    /// }

    /// void loop() {
    ///   Serial.write("Hello\n");
    ///   String s = "";
    ///   while (s.length() == 0) {
    ///     s = Serial.readString();
    ///   }
    ///   Serial.write("Goodbye ");
    ///   Serial.write(s.c_str());
    ///   Serial.write("\n");
    /// }
    ///```
    ///
    /// Remove the #[ignore] attribute and set the correct COM port to execute it.
	#[test]
	#[ignore]
	fn should_read_and_write() {
	    let mut iocp = CompletionPort::new().unwrap();	    
		let mut port = Serial::open_arduino("COM3", 9600).unwrap();
		port.set_timeouts(&SerialTimeouts::WaitToFill).unwrap();
		iocp.attach(EchoHandler::new(port.as_device())).unwrap();
		
		// Wait for Arduino to completely boot
		thread::sleep(time::Duration::from_millis(2000));
		
		loop {
		    if iocp.process_event(&time::Duration::from_millis(100)).is_err() {
		        break
		    }
		}
	}
	
	struct EchoHandler {
	    dev: Device,
	    hello_received: bool,
	}
	
	impl EchoHandler {
	    fn new(dev: Device) -> EchoHandler {
	        EchoHandler { dev: dev, hello_received: false }
	    }
	}
	
	impl DeviceHandler for EchoHandler {

		fn device(&mut self) -> &mut Device { &mut self.dev }
	    
	    fn process_event(&mut self, event: Event) -> io::Result<()> {
	        match event {
	            Event::Ready => self.dev.request_read_bytes(6),
	            Event::BytesRead(nbytes) if !self.hello_received => {
	                assert_eq!(nbytes, 6);
	                assert_eq!(self.dev.recv_bytes(), b"Hello\n");
	                self.dev.consume_recv_buffer(6);
	                self.hello_received = true;
	                self.dev.request_write(b"FlightVars")
	            }
	            Event::BytesWritten(nbytes) => {
	                assert_eq!(nbytes, 10);
	                self.dev.request_read_bytes(19)
	            }	            
	            Event::BytesRead(nbytes) if self.hello_received => {
	                assert_eq!(nbytes, 19);
	                assert_eq!(self.dev.recv_bytes(), b"Goodbye FlightVars\n");
	                self.dev.close()
	            }
	            _ => unreachable!(),
	        }
	    }
	}
}
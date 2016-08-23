//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::boxed::Box;
use std::collections::HashMap;
use std::io;
use std::time::Duration;

use types::*;

use super::device::*;
use super::ffi::*;

pub struct CompletionPort<H: DeviceHandler> {
    handle: HANDLE,
    devices: HashMap<DeviceId, Device>,
    handlers: HashMap<DeviceId, H>,
}

impl<H: DeviceHandler> CompletionPort<H> {
    pub fn new() -> io::Result<CompletionPort<H>> {
        let handle = checked_handle! { not_null =>
            CreateIoCompletionPort(
                INVALID_HANDLE_VALUE,
                0 as HANDLE,
                0 as ULONG_PTR,
                0)
        };
        Ok(CompletionPort {
            handle: handle,
            devices: HashMap::new(),
            handlers: HashMap::new(),
        })
    }
        
    pub fn attach(&mut self, dev: Device, handler: H) -> io::Result<DeviceId> {
        let handle = dev.handle();
        let id = dev.id();
        unsafe {            
            let rc = CreateIoCompletionPort(
                handle,
                self.handle,
                id as ULONG_PTR,
                0);
            if rc == 0 as HANDLE {
                return Err(io::Error::last_os_error());
            }
        }
        self.devices.insert(id, dev);
        self.handlers.insert(id, handler);
        Ok(id)
    }
    
    pub fn detach(&mut self, id: DeviceId) -> Option<Device> {
        self.devices.remove(&id)
    }
    
    pub fn device(&mut self, id: DeviceId) -> Option<&mut Device> {
        self.devices.get_mut(&id)
    }
    
    pub fn process_event(&mut self, timeout: &Duration) -> io::Result<DeviceId> {
        let mut nbytes: DWORD = 0;
        let mut key: ULONG_PTR = 0 as ULONG_PTR;
        let mut overlapped: LPOVERLAPPED = 0 as LPOVERLAPPED;
        let timeout_millis = 
        	(timeout.as_secs() as DWORD * 1000) + 
        	(timeout.subsec_nanos() as DWORD / 1000000);
        unsafe {
            let rc = GetQueuedCompletionStatus(
        		self.handle,
             	&mut nbytes as LPDWORD,
             	&mut key as PULONG_PTR,
             	&mut overlapped as *mut LPOVERLAPPED,
          	 	timeout_millis);
            if rc == 0 {
                return Err(io::Error::last_os_error());
            }
        };
        let id = key as DeviceId;
        let was_closed = {
            let dev = self.devices.get_mut(&id).unwrap();
            let handler = self.handlers.get_mut(&id).unwrap();
            if let Some(event) = dev.process_event() {
            	handler.process_event(dev, event);        
            }
          	dev.is_closed()  
        };
        if was_closed {
            self.devices.remove(&id);
            self.handlers.remove(&id);
        }
        Ok(id)
    }
}

#[cfg(test)]
mod test {
    
    use std::fs::File;
	use std::io::{Read, Write};
	use std::path::Path;
	use std::time::Duration;

    use tempdir::TempDir;

	use io::device::*;

    use super::*;
	
	#[test]
	fn should_read_device() {
	    with_file_content("should_read_device", "This is a file with some content", |path| {
		    let mut iocp = CompletionPort::new().unwrap();
	        let file = Device::open(path).unwrap();
	        let id = iocp.attach(file, |dev: &mut Device, _| {
                assert_eq!(dev.recv_bytes(), b"This is a file with some content");
	        }).unwrap();
		    iocp.device(id).unwrap().request_read().expect("request read");
		    let dev = iocp.process_event(&Duration::from_millis(100)).unwrap();
		    assert_eq!(dev, id);
		    iocp.device(id).unwrap().process_event();
        });
	}
	
	#[test]
	fn should_write_device() {
	    with_file_content("should_write_device", "", |path| {
        	{
    		    let mut iocp = CompletionPort::new().unwrap();
    	        let file = Device::open(path).unwrap();
    	        let id = iocp.attach(file, |_: &mut Device, event| {
	                if let Event::BytesWritten(n) = event {
		                assert_eq!(n, 32);	                    
	                } else {
	                    panic!("invalid written bytes count");
	                }
    	        }).unwrap();
    		    iocp.device(id).unwrap().request_write(b"This is a file with some content").unwrap();
    		    let dev = iocp.process_event(&Duration::from_millis(100)).unwrap();
    		    assert_eq!(dev, id);    		    
    		    iocp.device(id).unwrap().process_event();
    		    iocp.detach(id).unwrap().close().unwrap();
	    	}
		    assert_file_contains(path, "This is a file with some content");		    
        });
	}
	
	#[test]
	fn should_write_device_concurrently() {
	    with_file_content("should_write_device_concurrently", "", |path| {
	        let mut bytes_written = 0;
	    	{
    		    let mut iocp = CompletionPort::new().unwrap();
    	        let file = Device::open(path).unwrap();
    	        let id = iocp.attach(file, |_: &mut Device, event| {
	                if let Event::BytesWritten(n) = event {
		                bytes_written += n;	                    
	                } else {
	                    panic!("invalid written bytes count");
	                }
    	        }).unwrap();
    		    iocp.device(id).unwrap().request_write(b"This is a file with some content").expect("request write");
    		    iocp.device(id).unwrap().request_write(b"This is Sparta").expect("request write");

    		    let dev = iocp.process_event(&Duration::from_millis(100)).unwrap();
    		    assert_eq!(dev, id);
    		    iocp.device(id).unwrap().process_event();

    		    let dev = iocp.process_event(&Duration::from_millis(100)).unwrap();
    		    assert_eq!(dev, id);
    		    iocp.device(id).unwrap().process_event();
    		    iocp.detach(id).unwrap().close().unwrap();
	    	} 
        	assert_eq!(bytes_written, 46);
		    // Since no offset is given when writing, both writes will start at the begining
		    // of the file. This is pointless for serial and network ports. 
		    assert_file_contains(path, "This is Sparta with some content");
        });
	}
	
	fn with_file_content<F: FnOnce(&Path)>(name: &str, content: &str, f: F) {
	    let tmp_dir = TempDir::new("fv").expect("create temp dir");
	    let file_path = tmp_dir.path().join(name);
	    {
	    	let mut file = File::create(&file_path).expect("create temp file");
	    	write!(file, "{}", content).unwrap();
	    }
	    f(&file_path);
	}
	
	fn assert_file_contains(path: &Path, content: &str) {
	    let mut file = File::open(path).expect("open file");
	    let mut line = String::new();
	    file.read_to_string(&mut line).expect("read file");
	    assert_eq!(line, content);
	}	
}
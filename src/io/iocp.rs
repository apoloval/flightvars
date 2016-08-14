//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashSet;
use std::io;

use super::device::*;
use super::ffi::*;

pub struct CompletionPort {
    handle: HANDLE,
    next_key: ULONG_PTR,
    devices: HashSet<DeviceId>,
}

impl CompletionPort {
    pub fn new() -> io::Result<CompletionPort> {
        let handle = unsafe {
            CreateIoCompletionPort(
                INVALID_HANDLE_VALUE,
                0 as HANDLE,
                0 as ULONG_PTR,
                0)
        };
        if handle == 0 as HANDLE {
            return Err(io::Error::last_os_error());
        }
        Ok(CompletionPort {
            handle: handle,
            next_key: 1 as ULONG_PTR,
            devices: HashSet::new(),
        })
    }
        
    pub fn attach(&mut self, dev: &Device) -> io::Result<()> {
        let handle = dev.handle();
        let id = dev.id();
        let key = id.as_raw();
        self.next_key = (self.next_key as DWORD + 1) as ULONG_PTR;
        self.devices.insert(id);
        unsafe {            
            let rc = CreateIoCompletionPort(
                handle,
                self.handle,
                key as ULONG_PTR,
                0);
            if rc == 0 as HANDLE {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }
    
    pub fn process_event(&mut self) -> io::Result<DeviceId> {
        let mut nbytes: DWORD = 0;
        let mut key: ULONG_PTR = 0 as ULONG_PTR;
        let mut overlapped: LPOVERLAPPED = 0 as LPOVERLAPPED;
        unsafe {
            let rc = GetQueuedCompletionStatus(
        		self.handle,
             	&mut nbytes as LPDWORD,
             	&mut key as PULONG_PTR,
             	&mut overlapped as *mut LPOVERLAPPED,
          	 	100);
            if rc == 0 {
                return Err(io::Error::last_os_error());
            }
        };
        Ok(DeviceId::from_raw(key))
    }
}

#[cfg(test)]
mod test {
    
    use std::fs::File;
	use std::io::{Read, Write};
	use std::path::Path;

    use tempdir::TempDir;

	use io::device::*;

    use super::*;
	
	#[test]
	fn should_read_device() {
	    with_file_content("should_read_device", "This is a file with some content", |path| {
		    let mut iocp = CompletionPort::new().unwrap();
	        let mut file = Device::open(path).unwrap();
	        iocp.attach(&file).unwrap();
		    file.request_read().expect("request read");
		    let dev = iocp.process_event().unwrap();
		    assert_eq!(dev, file.id());
		    let event = file.process_event();
		    assert_eq!(event, Event::BytesRead(32));
		    assert_eq!(file.recv_bytes(), b"This is a file with some content"); 
        });
	}
	
	#[test]
	fn should_write_device() {
	    with_file_content("should_write_device", "", |path| {
		    let mut iocp = CompletionPort::new().unwrap();
	        let mut file = Device::open(path).unwrap();
	        iocp.attach(&file).unwrap();
		    file.request_write(b"This is a file with some content").expect("request write");
		    let dev = iocp.process_event().unwrap();
		    assert_eq!(dev, file.id());
		    let event = file.process_event();
		    assert_eq!(event, Event::BytesWritten(32));
		    file.close().expect("close file");
		    assert_file_contains(path, "This is a file with some content"); 
        });
	}
	
	#[test]
	fn should_write_device_concurrently() {
	    with_file_content("should_write_device_concurrently", "", |path| {
		    let mut iocp = CompletionPort::new().unwrap();
	        let mut file = Device::open(path).unwrap();
	        iocp.attach(&file).unwrap();
		    file.request_write(b"This is a file with some content").expect("request write");
		    file.request_write(b"This is Sparta").expect("request write");
		    let dev = iocp.process_event().unwrap();
		    assert_eq!(dev, file.id());
		    let event = file.process_event();
		    assert_eq!(event, Event::BytesWritten(32));
		    let dev = iocp.process_event().unwrap();
		    assert_eq!(dev, file.id());
		    let event = file.process_event();
		    assert_eq!(event, Event::BytesWritten(14));
		    file.close().expect("close file");
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
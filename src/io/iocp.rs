//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::collections::HashMap;
use std::io;
use std::time::Duration;

use types::*;

use super::device::*;
use super::ffi::*;

pub struct CompletionPort<H: DeviceHandler> {
    handle: HANDLE,
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
            handlers: HashMap::new(),
        })
    }
    
    pub fn handler(&mut self, dev: &DeviceId) -> Option<&mut H> {
        self.handlers.get_mut(dev)
    }
        
    pub fn attach(&mut self, mut handler: H) -> io::Result<DeviceId> {
        let handle = handler.device().handle();
        let id = handler.device().id();
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
        try!(handler.process_event(Event::Ready));
        self.handlers.insert(id, handler);
        Ok(id)
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
            let handler = self.handlers.get_mut(&id).unwrap();
            if let Some(event) = handler.device().process_event() {
            	if let Err(e) = handler.process_event(event) {
            	    error!("unexpected error while processing IO event: {:?}", e);
            	    info!("closing connection to handle {} due to IO errors", id);
            	    try!(handler.device().close());
            	}        
            }
          	handler.device().is_closed()  
        };
        if was_closed {
            self.handlers.remove(&id);
        }
        Ok(id)
    }
    
    pub fn is_timeout_error(&self, error: &io::Error) -> bool {
        error.raw_os_error() == Some(258)
    }
}

#[cfg(test)]
mod test {
    
    use std::fs::File;
    use std::io;
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
	        iocp.attach(FileReader::new(file)).unwrap();
		    iocp.process_event(&Duration::from_millis(100)).unwrap();
        });
	}
	
	#[test]
	fn should_write_device() {
	    with_file_content("should_write_device", "", |path| {
		    let mut iocp = CompletionPort::new().unwrap();
	        let file = Device::open(path).unwrap();
	        iocp.attach(FileWriter::new(file)).unwrap();
		    iocp.process_event(&Duration::from_millis(100)).unwrap();
		    assert_file_contains(path, "This is a file with some content");		    
        });
	}
	
	#[test]
	fn should_write_device_concurrently() {
	    with_file_content("should_write_device_concurrently", "", |path| {
		    let mut iocp = CompletionPort::new().unwrap();
	        let file = Device::open(path).unwrap();
	        iocp.attach(ParallelFileWriter::new(file)).unwrap();
		    iocp.process_event(&Duration::from_millis(100)).unwrap();
		    iocp.process_event(&Duration::from_millis(100)).unwrap();
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
	
	struct FileReader { dev: Device }
	
	impl FileReader {
	    fn new(dev: Device) -> FileReader {
	        FileReader { dev: dev }
	    }
	}
	
	impl DeviceHandler for FileReader {
	    
	    fn device(&mut self) -> &mut Device { &mut self.dev }
	        
    	fn process_event(&mut self, event: Event) -> io::Result<()> {
    	    match event {
    	        Event::Ready => self.dev.request_read(), 
    	        Event::BytesRead(n) => {
    	            assert_eq!(n, 32);
    	            assert_eq!(self.dev.recv_bytes(), b"This is a file with some content");
    	            self.dev.close()
    	        }
    	        _ => Ok(()),
    	    }
    	}
	}
	
	
	struct FileWriter { dev: Device }
	
	impl FileWriter {
	    fn new(dev: Device) -> FileWriter {
	        FileWriter { dev: dev }
	    }
	}
	
	impl DeviceHandler for FileWriter {
	    
	    fn device(&mut self) -> &mut Device { &mut self.dev }
	        
    	fn process_event(&mut self, event: Event) -> io::Result<()> {
    	    match event {
    	        Event::Ready => self.dev.request_write(b"This is a file with some content"), 
    	        Event::BytesWritten(n) => {
    	            assert_eq!(n, 32);
    	            self.dev.close()
    	        }
    	        _ => Ok(()),
    	    }
    	}
	}
	
	
	struct ParallelFileWriter { dev: Device, written: usize }
	
	impl ParallelFileWriter {
	    fn new(dev: Device) -> ParallelFileWriter {
	        ParallelFileWriter { dev: dev, written: 0 }
	    }
	}
	
	impl DeviceHandler for ParallelFileWriter {
	    
	    fn device(&mut self) -> &mut Device { &mut self.dev }
	        
    	fn process_event(&mut self, event: Event) -> io::Result<()> {
    	    match event {
    	        Event::Ready => { 
    	            try!(self.dev.request_write(b"This is a file with some content")); 
    	            self.dev.request_write(b"This is Sparta")
    	        }
    	        Event::BytesWritten(n) if self.written == 0 => {
    	            assert_eq!(n, 32);
    	            self.written += n;
    	            Ok(())
    	        }
    	        Event::BytesWritten(n) if self.written > 0 => {
    	            assert_eq!(n, 14);
    	            self.dev.close()
    	        }
    	        _ => Ok(()),
    	    }
    	}
	}
}
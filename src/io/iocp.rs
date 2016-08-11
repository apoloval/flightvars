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
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use super::buffer::Buffer;
use super::ffi::*;

pub struct CompletionPort<C> {
    handle: HANDLE,
    next_key: ULONG_PTR,
    devices: HashMap<ULONG_PTR, (Device, Box<DeviceHandler<C>>)>,
}

impl<C> CompletionPort<C> {
    pub fn new() -> io::Result<CompletionPort<C>> {
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
            devices: HashMap::new(),
        })
    }
        
    pub fn attach<H>(&mut self, dev: Device, handler: H) -> io::Result<&mut Device> 
    where H: DeviceHandler<C> + 'static {
        let handle = dev.handle;
        let key = self.next_key;
        self.next_key = (self.next_key as DWORD + 1) as ULONG_PTR;
        self.devices.insert(key, (dev, Box::new(handler)));
        let dev_ref = &mut self.devices.get_mut(&key).unwrap().0;
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
        Ok(dev_ref)
    }
    
    pub fn process_event(&mut self, context: &mut C) -> io::Result<()> {
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
        let dev_info = self.devices.get_mut(&key).unwrap();
        let dev = &mut dev_info.0;
        let handler = &mut dev_info.1;
        dev.read_buffer.extend(nbytes as usize);
        dev.read_pending = false;
        handler.process_read(context, dev);
        Ok(())
    }
}

pub struct Device {
    handle: HANDLE,
    read_overlapped: OVERLAPPED,
    read_buffer: Buffer,
    read_pending: bool,
}

impl Device {
    
    pub fn open(path: &Path) -> io::Result<Device> {
        let encoded_path: Vec<u16> = path
        	.as_os_str()
        	.encode_wide()
        	.chain(Some(0).into_iter())
        	.collect();
        let handle = unsafe {
            CreateFileW(
                encoded_path.as_ptr() as LPCWSTR,
          		GENERIC_ALL,
          		0,
          		0 as LPSECURITY_ATTRIBUTES,
           		OPEN_EXISTING,
          		FILE_FLAG_OVERLAPPED,
          		0 as HANDLE)
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        Ok(Device::from_handle(handle))
    }
    
    pub fn from_handle(handle: HANDLE) -> Device {
      Device {
          handle: handle,
          read_overlapped: OVERLAPPED::new(),
          read_buffer: Buffer::with_capacity(4096),
          read_pending: false,
      }  
    }
    
    pub fn read_buffer(&self) -> &[u8] {
        self.read_buffer.as_slice()
    }
    
    pub fn request_read(&mut self) -> io::Result<()>{
        assert!(!self.read_pending);
        let rc = unsafe {
            ReadFile(
                self.handle,
                self.read_buffer.as_mut_ptr() as LPVOID,
                self.read_buffer.remaining() as DWORD,
                0 as LPDWORD,
                &mut self.read_overlapped as LPOVERLAPPED)
        };
        if rc == 0 && unsafe { GetLastError() } != ERROR_IO_PENDING {
            return Err(io::Error::last_os_error());
        }
        self.read_pending = true;
        Ok(())
    }
}

pub trait DeviceHandler<C> {
    fn process_read(&mut self, context: &mut C, dev: &mut Device);
}

#[cfg(test)]
mod test {
    
    use std::fs::File;
	use std::io::Write;
	use std::path::Path;

    use tempdir::TempDir;

    use super::*;
	
	#[test]
	fn should_read_device() {
	    with_file_content("foobar", "This is a file with some content", |path| {
            let mut context = "context: ".to_string();
		    let mut iocp = CompletionPort::new().unwrap();
		    {
		        let file = Device::open(path).unwrap();
		        let dev = iocp.attach(file, MockDeviceHandler).unwrap();
    		    dev.request_read().expect("request read");
		    }
		    iocp.process_event(&mut context).unwrap();
		    assert_eq!(context, "This is a file with some content"); 
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
	
	struct MockDeviceHandler;
	
	impl DeviceHandler<String> for MockDeviceHandler {
	    fn process_read(&mut self, context: &mut String, dev: &mut Device) {
	    	*context = String::from_utf8(dev.read_buffer().to_vec()).unwrap();
	    }
	}
}
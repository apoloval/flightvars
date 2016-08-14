//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::hash::{Hash, Hasher};
use std::io;
use std::io::Write;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use super::buffer::Buffer;
use super::ffi::*;

#[derive(Debug, Eq, PartialEq)]
pub struct DeviceId(ULONG_PTR);

impl DeviceId {
    pub fn from_raw(raw: ULONG_PTR) -> DeviceId {
        DeviceId(raw)
    }
    
    pub fn as_raw(&self) -> ULONG_PTR { self.0 }
}

impl Hash for DeviceId {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.0.hash(state)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Event {
    None,
    BytesRead(usize),
    BytesWritten(usize),
}

pub struct Device {
    handle: HANDLE,
    read_overlapped: OVERLAPPED,
    read_buffer: Buffer,
    read_pending: bool,
    write_overlapped: OVERLAPPED,
    write_buffer: Buffer,
    write_pending: bool,
}

impl Device {
    
    pub fn id(&self) -> DeviceId {
        DeviceId(self.handle as ULONG_PTR)
    }
    
    pub fn handle(&self) -> HANDLE { self.handle }
    
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
          write_overlapped: OVERLAPPED::new(),
          write_buffer: Buffer::with_capacity(4096),
          write_pending: false,
      }  
    }
    
    pub fn close(self) -> io::Result<()> {
        let rc = unsafe { 
            CloseHandle(self.handle) 
        };
        if rc == 0 { Err(io::Error::last_os_error()) } 
        else { Ok(()) }
    }
    
    pub fn recv_bytes(&self) -> &[u8] {
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
    
    pub fn request_write(&mut self, data: &[u8]) -> io::Result<()> {
        try!(self.write_buffer.write(data));
        assert!(!self.write_pending);
        let rc = unsafe {
            WriteFile(
                self.handle,
                self.write_buffer.as_ptr() as LPCVOID,
                self.write_buffer.len() as DWORD,
                0 as LPDWORD,
                &mut self.write_overlapped as LPOVERLAPPED)
        };
        if rc == 0 && unsafe { GetLastError() } != ERROR_IO_PENDING {
            return Err(io::Error::last_os_error());
        }
        self.write_pending = true;
        Ok(())
    }
    
    pub fn process_event(&mut self) -> Event {
        if self.read_event_ready() {
        	Event::BytesRead(self.process_read_event())
        } else if self.write_event_ready() {
            Event::BytesWritten(self.process_write_event())
        } else {
            Event::None
        }
    }
    
    fn read_event_ready(&self) -> bool  {
        self.read_pending && self.read_overlapped.Internal != STATUS_PENDING
    }
    
    fn write_event_ready(&self) -> bool  {
        self.write_pending && self.write_overlapped.Internal != STATUS_PENDING
    }
    
    fn process_read_event(&mut self) -> usize {
        assert!(self.read_event_ready());
        let nbytes = self.read_overlapped.InternalHigh as usize;
        self.read_buffer.extend(nbytes);
        self.read_pending = false;
        nbytes
    }

    fn process_write_event(&mut self) -> usize {
        assert!(self.write_event_ready());
        let nbytes = self.write_overlapped.InternalHigh as usize;
        self.write_buffer.clear();
        self.write_pending = false;
        nbytes
    }
}

//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::hash::{Hash, Hasher};
use std::io;
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
}

pub struct Device {
    handle: HANDLE,
    read_overlapped: OVERLAPPED,
    read_buffer: Buffer,
    read_pending: bool,
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
      }  
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
    
    pub fn process_event(&mut self) -> Event {
        self.process_read_event()
    }
    
    pub fn process_read_event(&mut self) -> Event {
        if self.read_pending && self.read_overlapped.Internal != STATUS_PENDING {
            let nbytes = self.read_overlapped.InternalHigh as usize;
            self.read_buffer.extend(nbytes);
            self.read_pending = false;
            Event::BytesRead(nbytes)
        } else {
            Event::None
        }
    }
}

//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::boxed::Box;
use std::cmp;
use std::io;
use std::io::Write;
use std::ops::DerefMut;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use types::*;

use super::buffer::Buffer;
use super::ffi::*;

#[derive(Debug, Eq, PartialEq)]
pub enum Event {
    Ready,
    BytesRead(usize),
    BytesWritten(usize),
}

pub struct DeviceControlBlock {
    buffer: Buffer,
    overlapped: OVERLAPPED,
}

impl DeviceControlBlock {
    pub fn new() -> DeviceControlBlock {
        DeviceControlBlock {
            buffer: Buffer::with_capacity(4096),
            overlapped: OVERLAPPED::new(),
        }
    }
}

pub struct Device {
    handle: HANDLE,
    read_control_block: DeviceControlBlock,
    read_pending: bool,
    write_control_blocks: Vec<Box<DeviceControlBlock>>,
}

impl Device {
    
    pub fn id(&self) -> DeviceId {
        self.handle as DeviceId
    }
    
    pub fn handle(&self) -> HANDLE { self.handle }

	#[allow(dead_code)]    
    pub fn open(path: &Path) -> io::Result<Device> {
        let encoded_path: Vec<u16> = path
        	.as_os_str()
        	.encode_wide()
        	.chain(Some(0).into_iter())
        	.collect();
        let handle = checked_handle! { valid =>
            CreateFileW(
                encoded_path.as_ptr() as LPCWSTR,
          		GENERIC_ALL,
          		0,
          		0 as LPSECURITY_ATTRIBUTES,
           		OPEN_EXISTING,
          		FILE_FLAG_OVERLAPPED,
          		0 as HANDLE)
        };
        Ok(Device::from_handle(handle))
    }
    
    pub fn from_handle(handle: HANDLE) -> Device {
      Device {
          handle: handle,
          read_control_block: DeviceControlBlock::new(),
          read_pending: false,
          write_control_blocks: Vec::with_capacity(32),
      }  
    }
    
    pub fn close(&mut self) -> io::Result<()> {
        let rc = unsafe { 
            CloseHandle(self.handle) 
        };
        if rc == 0 { Err(io::Error::last_os_error()) } 
        else {
            self.handle = INVALID_HANDLE_VALUE; 
            Ok(()) 
        }            
    }
    
    pub fn is_closed(&self) -> bool {
        self.handle == INVALID_HANDLE_VALUE
    }
    
    pub fn consume_recv_buffer(&mut self, nbytes: usize) {
        self.read_control_block.buffer.consume(nbytes)
    }
    
    pub fn recv_bytes(&self) -> &[u8] {
        self.read_control_block.buffer.as_slice()
    }
    
    pub fn request_read(&mut self) -> io::Result<()>{
        let nbytes = self.read_control_block.buffer.remaining();
        self.request_read_bytes(nbytes)
    }
    
    pub fn request_read_bytes(&mut self, nbytes: usize) -> io::Result<()>{
        assert!(!self.read_pending);
        let total_bytes = cmp::min(nbytes, self.read_control_block.buffer.remaining());
        let offset = self.read_control_block.buffer.len() as isize;
        let rc = unsafe {
            ReadFile(
                self.handle,
                self.read_control_block.buffer.as_mut_ptr().offset(offset) as LPVOID,
                total_bytes as DWORD,
                0 as LPDWORD,
                &mut self.read_control_block.overlapped as LPOVERLAPPED)
        };
        if rc == 0 && unsafe { GetLastError() } != ERROR_IO_PENDING {
            return Err(io::Error::last_os_error());
        }
        self.read_pending = true;
        Ok(())
    }
    
    pub fn request_write(&mut self, data: &[u8]) -> io::Result<()> {
        self.write_control_blocks.push(Box::new(DeviceControlBlock::new()));
        let cb = self.write_control_blocks.last_mut().unwrap();
        try!(cb.buffer.write(data));
        let rc = unsafe {
            WriteFile(
                self.handle,
                cb.buffer.as_ptr() as LPCVOID,
                cb.buffer.len() as DWORD,
                0 as LPDWORD,
                &mut cb.overlapped as LPOVERLAPPED)
        };
        if rc == 0 && unsafe { GetLastError() } != ERROR_IO_PENDING {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
    
    pub fn process_event(&mut self) -> Option<Event> {
        if self.read_event_ready() {
        	Some(Event::BytesRead(self.process_read_event()))
        } else if self.write_event_ready() {
            Some(Event::BytesWritten(self.process_write_event()))
        } else {
            None
        }
    }
    
    pub fn read_event_ready(&self) -> bool  {
        self.read_pending && self.read_control_block.overlapped.Internal != STATUS_PENDING
    }
    
    pub fn write_event_ready(&self) -> bool  {
        self.write_control_blocks
        	.first()
        	.iter()
        	.any(|cb| cb.overlapped.Internal != STATUS_PENDING)
    }
    
    fn process_read_event(&mut self) -> usize {
        assert!(self.read_event_ready());
        let nbytes = self.read_control_block.overlapped.InternalHigh as usize;
        self.read_control_block.buffer.extend(nbytes);
        self.read_pending = false;
        nbytes
    }

    fn process_write_event(&mut self) -> usize {
        assert!(self.write_event_ready());
        let cb = self.write_control_blocks.remove(0);
        cb.overlapped.InternalHigh as usize
    }
}

pub trait DeviceHandler {    
    fn device(&mut self) -> &mut Device;    
    fn process_event(&mut self, event: Event) -> io::Result<()>;    
}

impl<H: DeviceHandler + ?Sized> DeviceHandler for Box<H> {
    fn device(&mut self) -> &mut Device {
        self.deref_mut().device()
    }   
     
    fn process_event(&mut self, event: Event) -> io::Result<()> {
        self.deref_mut().process_event(event)
    }    
}

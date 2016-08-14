//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;

pub struct Buffer {
    data: Vec<u8>,
}

impl Buffer {
    
    pub fn with_capacity(capacity: usize) -> Buffer {
        Buffer { data: Vec::with_capacity(capacity) }
    }
    
    pub fn len(&self) -> usize { self.data.len() }
    
    pub fn remaining(&self) -> usize { self.data.capacity() - self.data.len() } 
    
    pub fn as_slice(&self) -> &[u8] { self.data.as_slice() }
    
    pub fn as_ptr(&mut self) -> *const u8 { self.data.as_ptr() }

    pub fn as_mut_ptr(&mut self) -> *mut u8 { self.data.as_mut_ptr() }
    
    pub fn extend(&mut self, inc: usize) {
        let new_len = self.data.len() + inc;
        assert!(new_len < self.data.capacity());
        unsafe { self.data.set_len(new_len) }; 
    }
    
    pub fn clear(&mut self) {
        self.data.clear()
    }
}

impl io::Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.data.write(buf)
    }
    
    fn flush(&mut self) -> io::Result<()> {
        self.data.flush()
    }
}

#[cfg(test)]
mod test {
    
    use super::*;
    
    #[test]
    fn should_init_empty() {
        let buf = Buffer::with_capacity(16);
        assert_eq!(buf.len(), 0);
    }
    
    #[test]
    fn should_extend() {
        let mut buf = Buffer::with_capacity(16);
        buf.extend(4);
        assert_eq!(buf.len(), 4);
        buf.extend(2);
        assert_eq!(buf.len(), 6);
    }
    
    #[test]
    #[should_panic]
    fn should_panic_extend_beyond_capacity() {
        let mut buf = Buffer::with_capacity(16);
        buf.extend(20);
    }
}
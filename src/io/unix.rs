//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::os::unix::io::{IntoRawFd, RawFd};
use std::sync::Arc;

use libc;

pub fn split<T: IntoRawFd + io::Read + io::Write>(device: T) -> (SplitRead, SplitWrite) {
    let fd = device.into_raw_fd();
    let socket = Arc::new(SharedFd::new(fd));
    (SplitRead::new(&socket), SplitWrite::new(&socket))
}

#[link(name = "c")]
extern {
    static mut errno: libc::c_int;
}

struct SharedFd {
    fd: RawFd
}

impl SharedFd {
    fn new(fd: RawFd) -> SharedFd { SharedFd { fd: fd }}
}

impl Drop for SharedFd {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd); }
    }
}

pub struct SplitRead {
    socket: Arc<SharedFd>
}

impl SplitRead {
    fn new(socket: &Arc<SharedFd>) -> SplitRead { SplitRead { socket: socket.clone() }}
}

impl io::Read for SplitRead {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let c_buf = buf as *mut [u8] as *mut libc::c_void;
            let bytes_read = libc::read(self.socket.fd, c_buf, buf.len() as u64);
            if bytes_read >= 0 { Ok(bytes_read as usize) }
            else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("cannot read from device (errno {})", errno)))
            }
        }
    }
}

pub struct SplitWrite {
    socket: Arc<SharedFd>
}

impl SplitWrite {
    fn new(socket: &Arc<SharedFd>) -> SplitWrite { SplitWrite { socket: socket.clone() }}
}

impl io::Write for SplitWrite {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unsafe {
            let c_buf = buf as *const [u8] as *const libc::c_void;
            let bytes_written = libc::write(self.socket.fd, c_buf, buf.len() as u64);
            if bytes_written >= 0 { Ok(bytes_written as usize) }
            else { Err(io::Error::new(
                io::ErrorKind::Other,
                format!("cannot write to device (errno {})", errno)))
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        unsafe {
            if libc::fsync(self.socket.fd) == 0 { Ok(()) }
            else { Err(io::Error::new(
                io::ErrorKind::Other,
                format!("cannot sync to device (errno {})", errno)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::*;
    use std::net::TcpStream;
    use std::thread;

    use super::*;

    #[test]
    fn should_read_and_write_after_split() {
        let stream = TcpStream::connect("www.google.com:80").unwrap();
        let (mut input, mut output) = split(stream);
        output.write_all(b"GET / HTTP/1.1\n").unwrap();
        output.write_all(b"Host: www.google.com\n").unwrap();
        output.write_all(b"Connetion: close\n\n").unwrap();
        let mut rep = [0; 4];
        assert_eq!(input.read(&mut rep).unwrap(), 4);
        assert_eq!(&rep[..], b"HTTP");
    }

    #[test]
    fn should_read_after_write_is_drop() {
        let stream = TcpStream::connect("www.google.com:80").unwrap();
        let (mut input, output) = split(stream);
        {
            let mut output = output;
            output.write_all(b"GET / HTTP/1.1\n").unwrap();
            output.write_all(b"Host: www.google.com\n").unwrap();
            output.write_all(b"Connetion: close\n\n").unwrap();
        }
        let mut rep = [0; 4];
        assert_eq!(input.read(&mut rep).unwrap(), 4);
        assert_eq!(&rep[..], b"HTTP");
    }

    #[test]
    fn should_read_and_write_after_split_from_different_threads() {
        let stream = TcpStream::connect("www.google.com:80").unwrap();
        let (mut input, mut output) = split(stream);
        let th1 = thread::spawn(move || {
            output.write_all(b"GET / HTTP/1.1\n").unwrap();
            output.write_all(b"Host: www.google.com\n").unwrap();
            output.write_all(b"Connetion: close\n\n").unwrap();
        });
        let th2 = thread::spawn(move || {
            let mut rep = [0; 4];
            assert_eq!(input.read(&mut rep).unwrap(), 4);
            assert_eq!(&rep[..], b"HTTP");
        });
        th1.join().unwrap();
        th2.join().unwrap();
    }


}

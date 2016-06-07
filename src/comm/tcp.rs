//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::net;
use std::time::Duration;

use comm::*;

const READ_TIMEOUT: u64 = 250;

#[cfg(unix)]
impl io::Read for TcpInput {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf).map_err(|e|
            if e.kind() == io::ErrorKind::Other && e.raw_os_error() == Some(9) {
                io::Error::new(io::ErrorKind::ConnectionAborted, "tcp input was interrupted")
            } else { e }
        )
    }
}


pub struct TcpListener(net::TcpListener);

impl TcpListener {
    pub fn bind<A: net::ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
        net::TcpListener::bind(addr).map(|l| TcpListener(l))
    }

    #[cfg(unix)]
    fn accept(&mut self) -> io::Result<(net::TcpStream, net::SocketAddr)> {
        self.0.accept()
    }

    #[cfg(windows)]
    fn accept(&mut self) -> io::Result<(net::TcpStream, net::SocketAddr)> {
        self.0.accept()
            .map_err(Self::map_interrupt_error)
    }

    #[cfg(windows)]
    fn map_interrupt_error(error: io::Error) -> io::Error {
        if error.kind() == io::ErrorKind::Other && error.raw_os_error() == Some(10004) {
            io::Error::new(io::ErrorKind::ConnectionAborted, "listener was interrupted")
        } else {
            error
        }
    }
}

impl Listen for TcpListener {
	type ConnAddr = net::SocketAddr;
    type Input = net::TcpStream;
    type Output = net::TcpStream;
    type Int = TcpInterruptor;

    fn listen(&mut self) -> io::Result<(net::TcpStream, net::TcpStream, net::SocketAddr)> {
        let (input, addr) = try!(self.accept());
        let output = try!(input.try_clone());
        try!(input.set_read_timeout(Some(Duration::from_millis(READ_TIMEOUT))));
        Ok((input, output, addr))
    }

    fn shutdown_interruption(&mut self) -> TcpInterruptor {
        TcpInterruptor::from_listener(&self.0)
    }
}


#[cfg(unix)]
pub mod unix {
    use std::net;
    use std::os::unix::io::{AsRawFd, RawFd};

    use libc;

    use comm::*;

    pub struct TcpInterruptor {
        fd: RawFd
    }

    impl TcpInterruptor {
        pub fn from_listener(listener: &net::TcpListener) -> TcpInterruptor {
            TcpInterruptor { fd: listener.as_raw_fd() }
        }
    }

    impl Interrupt for TcpInterruptor {
        fn interrupt(self) {
            unsafe { libc::close(self.fd); }
        }
    }
}

#[cfg(unix)]
pub use self::unix::*;

#[cfg(windows)]
pub mod win {
    use std::net;
    use std::os::windows::io::{AsRawSocket, RawSocket};

    use ws2_32;

    use comm::*;

    pub struct TcpInterruptor {
        socket: RawSocket
    }

    impl TcpInterruptor {
        pub fn from_listener(listener: &net::TcpListener) -> TcpInterruptor {
            TcpInterruptor { socket: listener.as_raw_socket() }
        }
    }

    impl Interrupt for TcpInterruptor {
        fn interrupt(self) {
            unsafe {
                if ws2_32::closesocket(self.socket) != 0 {
                    let error_code = ws2_32::WSAGetLastError();
                    error!("unexpected error while closing TCP socket: error code {}", error_code);
                }
            }
        }
    }
}

#[cfg(windows)]
pub use self::win::*;

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::{BufRead, Write};
    use std::net;
    use std::thread;
    use std::time;

    use comm::*;

    use super::*;

    #[test]
    fn should_wait_conn() {
        let child = thread::spawn(|| {
            let mut listener = TcpListener::bind("127.0.0.1:1234").unwrap();
            let (r, mut w, _) = listener.listen().unwrap();
            let mut input = io::BufReader::new(r);
            let mut line = String::new();
            input.read_line(&mut line).unwrap();
            assert_eq!(line, "Hello server\n");
            w.write_all(b"Hello client\n").unwrap();
        });
        thread::sleep(time::Duration::from_millis(25));
        let mut conn = net::TcpStream::connect("127.0.0.1:1234").unwrap();
        conn.write_all(b"Hello server\n").unwrap();
        let mut input = io::BufReader::new(conn);
        let mut line = String::new();
        input.read_line(&mut line).unwrap();
        assert_eq!(line, "Hello client\n");
        child.join().unwrap();
    }

    #[test]
    fn should_close_listener_from_shutdown_handle() {
        let mut listener = TcpListener::bind("127.0.0.1:1235").unwrap();
        let interruption = listener.shutdown_interruption();
        let child = thread::spawn(move || {
            assert_eq!(listener.listen().unwrap_err().kind(), io::ErrorKind::ConnectionAborted);
        });
        thread::sleep(time::Duration::from_millis(25));
        interruption.interrupt();
        child.join().unwrap();
    }    
}

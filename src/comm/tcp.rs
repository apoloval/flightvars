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

#[derive(Debug)]
pub struct TcpInput(net::TcpStream);

impl TcpInput {
	pub fn new(stream: net::TcpStream) -> io::Result<TcpInput> {
		try!(stream.set_read_timeout(Some(Duration::from_millis(READ_TIMEOUT))));
		Ok(TcpInput(stream))
	}
}

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

#[cfg(windows)]
impl io::Read for TcpInput {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl ShutdownInterruption for TcpInput {
    type Int = TcpInterruptor;

    fn shutdown_interruption(&mut self) -> TcpInterruptor {
        TcpInterruptor::from_stream(&self.0)
    }
}

impl Identify for TcpInput {
    fn id(&self) -> String {
        self.0.peer_addr()
            .map(|a| format!("{}", a))
            .unwrap_or_else(|_| format!("{:?}", self.0))
    }
}

#[derive(Debug)]
pub struct TcpOutput(net::TcpStream);

impl io::Write for TcpOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> { self.0.write(buf) }
    fn flush(&mut self) -> io::Result<()> { self.0.flush() }
}


pub struct TcpListener(net::TcpListener);

impl TcpListener {
    pub fn bind<A: net::ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
        net::TcpListener::bind(addr).map(|l| TcpListener(l))
    }

    #[cfg(unix)]
    fn accept(&mut self) -> io::Result<net::TcpStream> {
        self.0.accept().map(|(s, _)| s)
    }

    #[cfg(windows)]
    fn accept(&mut self) -> io::Result<net::TcpStream> {
        self.0.accept()
            .map(|(s, _)| s)
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

impl Listen<TcpInput, TcpOutput> for TcpListener {
    fn listen(&mut self) -> io::Result<(TcpInput, TcpOutput)> {
        let conn = try!(self.accept());
        let input = try!(TcpInput::new(try!(conn.try_clone())));
        let output = TcpOutput(conn);
        Ok((input, output))
    }
}

impl ShutdownInterruption for TcpListener {
    type Int = TcpInterruptor;

    fn shutdown_interruption(&mut self) -> TcpInterruptor {
        TcpInterruptor::from_listener(&self.0)
    }
}


pub struct TcpTransport {
    listener: TcpListener
}

impl TcpTransport {
    pub fn bind<A: net::ToSocketAddrs>(addr: A) -> io::Result<TcpTransport> {
        Ok(TcpTransport { listener: try!(TcpListener::bind(addr)) })
    }
}

impl Transport for TcpTransport {
    type Input = TcpInput;
    type Output = TcpOutput;
    type Listener = TcpListener;

    fn listener(&mut self) -> &mut TcpListener {
        &mut self.listener
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

        pub fn from_stream(stream: &net::TcpStream) -> TcpInterruptor {
            TcpInterruptor { fd: stream.as_raw_fd() }
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

        pub fn from_stream(stream: &net::TcpStream) -> TcpInterruptor {
            TcpInterruptor { socket: stream.as_raw_socket() }
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
    use std::io::{BufRead, Read, Write};
    use std::net;
    use std::thread;
    use std::time;

    use comm::*;

    use super::*;

    #[test]
    fn should_wait_conn() {
        let child = thread::spawn(|| {
            let mut listener = TcpListener::bind("127.0.0.1:1234").unwrap();
            let (r, mut w) = listener.listen().unwrap();
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

    #[test]
    fn should_close_input_from_shutdown_handle() {
        let mut listener = TcpListener::bind("127.0.0.1:1236").unwrap();
        let interruption = listener.shutdown_interruption();
        let client = thread::spawn(move || {
            let _conn = net::TcpStream::connect("127.0.0.1:1236");
            thread::sleep(time::Duration::from_millis(50));
        });
        let child = thread::spawn(move || {
            let (mut conn, _) = listener.listen().unwrap();
            let interruption = conn.shutdown_interruption();
            thread::spawn(move || {
                thread::sleep(time::Duration::from_millis(20));
                interruption.interrupt();
            });
            let mut buf = [0; 10];
            assert_eq!(conn.read(&mut buf).unwrap_err().kind(), io::ErrorKind::ConnectionAborted);
        });
        thread::sleep(time::Duration::from_millis(50));
        interruption.interrupt();
        child.join().unwrap();
        client.join().unwrap();
    }
}

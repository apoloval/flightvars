//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::net;

use comm::*;

pub struct TcpTransport {
    listener: net::TcpListener
}

impl TcpTransport {
    pub fn bind<A: net::ToSocketAddrs>(addr: A) -> io::Result<TcpTransport> {
        Ok(TcpTransport { listener: try!(net::TcpListener::bind(addr)) })
    }
}

impl Transport for TcpTransport {
    type Input = net::TcpStream;
    type Output = net::TcpStream;
    type Listener = net::TcpListener;

    fn listener(&mut self) -> &mut net::TcpListener {
        &mut self.listener
    }
}

impl Listen<net::TcpStream, net::TcpStream> for net::TcpListener {
    fn listen(&mut self) -> io::Result<(net::TcpStream, net::TcpStream)> {
        let (conn, _) = try!(self.accept());
        let input = try!(conn.try_clone());
        let output = conn;
        Ok((input, output))
    }
}

impl ShutdownInterruption for net::TcpListener {
    type Int = TcpInterruptor;

    fn shutdown_interruption(&mut self) -> TcpInterruptor {
        TcpInterruptor::from_listener(self)
    }
}

impl ShutdownInterruption for net::TcpStream {
    type Int = TcpInterruptor;

    fn shutdown_interruption(&mut self) -> TcpInterruptor {
        TcpInterruptor::from_stream(self)
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

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::{BufRead, Write};
    use std::net;
    use std::thread;
    use std::time;

    use comm::*;

    #[test]
    fn should_wait_conn() {
        let child = thread::spawn(|| {
            let mut listener = net::TcpListener::bind("127.0.0.1:1234").unwrap();
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
    fn should_close_from_shutdown_handle() {
        let mut listener = net::TcpListener::bind("127.0.0.1:1235").unwrap();
        let interruption = listener.shutdown_interruption();
        let child = thread::spawn(move || {
            assert_eq!(listener.listen().unwrap_err().kind(), io::ErrorKind::ConnectionAborted);
        });
        thread::sleep(time::Duration::from_millis(25));
        interruption.interrupt();
        child.join().unwrap();
    }
}

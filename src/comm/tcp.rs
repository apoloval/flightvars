//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::net;

use comm::Transport;

impl Transport for net::TcpListener {
    type Input = net::TcpStream;
    type Output = net::TcpStream;
    type Shutdown = TcpShutdownHandler;

    fn wait_conn(&mut self) -> io::Result<(net::TcpStream, net::TcpStream)> {
        let (read, _) = try!(self.accept());
        let write = try!(read.try_clone());
        Ok((read, write))
    }

    fn shutdown_handle(&mut self) -> Self::Shutdown {
        TcpShutdownHandler::from_listener(&self)
    }
}

#[cfg(unix)]
pub mod unix {
    use std::net;
    use std::os::unix::io::{AsRawFd, RawFd};

    use libc;

    use comm::ShutdownHandle;

    pub struct TcpShutdownHandler {
        fd: RawFd
    }

    impl TcpShutdownHandler {
        pub fn from_listener(listener: &net::TcpListener) -> TcpShutdownHandler {
            TcpShutdownHandler { fd: listener.as_raw_fd() }
        }
    }

    impl ShutdownHandle for TcpShutdownHandler {
        fn shutdown(self) {
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

    use comm::{ShutdownHandle, Transport};

    #[test]
    fn should_wait_conn() {
        let child = thread::spawn(|| {
            let mut listener = net::TcpListener::bind("127.0.0.1:1234").unwrap();
            let (r, mut w) = listener.wait_conn().unwrap();
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
        let handle = listener.shutdown_handle();
        let child = thread::spawn(move || {
            assert_eq!(listener.wait_conn().unwrap_err().kind(), io::ErrorKind::ConnectionAborted);
        });
        thread::sleep(time::Duration::from_millis(25));
        handle.shutdown();
        child.join().unwrap();
    }
}

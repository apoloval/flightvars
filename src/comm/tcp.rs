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

pub struct Tcp {
    listener: net::TcpListener
}

impl Tcp {
    fn bind<A: net::ToSocketAddrs>(addr: A) -> io::Result<Tcp> {
        Ok(Tcp { listener: try!(net::TcpListener::bind(addr)) })
    }
}


impl Transport for net::TcpListener {
    type Input = net::TcpStream;
    type Output = net::TcpStream;

    fn wait_conn(&mut self) -> io::Result<(net::TcpStream, net::TcpStream)> {
        let (read, _) = try!(self.accept());
        let write = try!(read.try_clone());
        Ok((read, write))
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::{BufRead, Write};
    use std::net;
    use std::thread;
    use std::time;

    use comm::Transport;

    use super::*;

    #[test]
    fn should_wait_conn() {
        let child = thread::spawn(|| {
            let mut listener = net::TcpListener::bind("127.0.0.1:1234").unwrap();
            let (mut r, mut w) = listener.wait_conn().unwrap();
            let mut input = io::BufReader::new(r);
            let mut line = String::new();
            input.read_line(&mut line);
            assert_eq!(line, "Hello server\n");
            w.write_all(b"Hello client\n");
        });
        thread::sleep(time::Duration::from_millis(25));
        let mut conn = net::TcpStream::connect("127.0.0.1:1234").unwrap();
        conn.write_all(b"Hello server\n");
        let mut input = io::BufReader::new(conn);
        let mut line = String::new();
        input.read_line(&mut line);
        assert_eq!(line, "Hello client\n");
        child.join().unwrap();
    }
}

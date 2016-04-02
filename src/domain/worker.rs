//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::time;

use domain::Command;
use domain::notify::*;

const POLLING_DELAY_MS: u64 = 20;

pub trait Handler {
    fn command(&mut self, cmd: Command);
    fn poll(&mut self);
}

pub struct Worker {
    tx: NotifySender<Envelope>,
    rx: NotifyReceiver<Envelope>,
    run: bool,
}

impl Worker {
    pub fn new() -> Worker {
        let (tx, rx) = notification();
        Worker {
            tx: tx,
            rx: rx,
            run: true,
        }
    }

    pub fn sender(&self) -> NotifySender<Envelope> {
        self.tx.clone()
    }

    pub fn run<H: Handler>(&mut self, handler: &mut H) {
        self.run = true;
        let timeout = time::Duration::from_millis(POLLING_DELAY_MS);
        while self.run {
            match self.rx.recv(timeout) {
                Ok(Some(Envelope::Shutdown)) => self.shutdown(),
                Ok(Some(Envelope::Cmd(cmd))) => handler.command(cmd),
                Ok(None) => handler.poll(),
                _ => {},
            }
        }
    }

    pub fn shutdown(&mut self) {
        self.run = false;
    }
}

#[derive(Debug)]
pub enum Envelope {
    Cmd(Command),
    Shutdown
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::thread;

    use domain::{Command, Var, Value};

    use super::*;

    #[test]
    fn should_shutdown() {
        let mut worker = Worker::new();
        let tx = worker.sender();
        let (polling_tx, _) = mpsc::channel();
        let (command_tx, _) = mpsc::channel();
        let child = thread::spawn(move || {
            let mut handler = MockHandler {
                pollings: polling_tx,
                commands: command_tx,
            };
            worker.run(&mut handler);
        });
        assert!(tx.send(Envelope::Shutdown).is_ok());
        assert!(child.join().is_ok());
    }

    #[test]
    fn should_tick_polling() {
        let mut worker = Worker::new();
        let tx = worker.sender();
        let (polling_tx, polling_rx) = mpsc::channel();
        let (command_tx, _) = mpsc::channel();
        let child = thread::spawn(move || {
            let mut handler = MockHandler {
                pollings: polling_tx,
                commands: command_tx,
            };
            worker.run(&mut handler);
        });
        assert!(polling_rx.recv().is_ok());
        assert!(tx.send(Envelope::Shutdown).is_ok());
        assert!(child.join().is_ok());
    }

    #[test]
    fn should_process_msg() {
        let mut worker = Worker::new();
        let tx = worker.sender();
        let (polling_tx, _polling_rx) = mpsc::channel();
        let (command_tx, command_rx) = mpsc::channel();
        let child = thread::spawn(move || {
            let mut handler = MockHandler {
                pollings: polling_tx,
                commands: command_tx,
            };
            worker.run(&mut handler);
        });
        let cmd = Command::Write(Var::lvar("foobar"), Value::Bool(true));
        assert!(tx.send(Envelope::Cmd(cmd.clone())).is_ok());
        assert_eq!(command_rx.recv().unwrap(), cmd);
        assert!(tx.send(Envelope::Shutdown).is_ok());
        assert!(child.join().is_ok());
    }

    struct MockHandler {
        pollings: mpsc::Sender<()>,
        commands: mpsc::Sender<Command>,
    }

    impl Handler for MockHandler {
        fn command(&mut self, cmd: Command) {
            self.commands.send(cmd).unwrap();
        }

        fn poll(&mut self) {
            self.pollings.send(()).unwrap();
        }
    }
}

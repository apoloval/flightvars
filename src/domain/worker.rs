//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::thread;
use std::time;

use domain::Command;
use domain::notify::*;
use util::Consume;

const POLLING_DELAY_MS: u64 = 20;

pub trait Handle {
    fn new() -> Self;
    fn description() -> String;
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

    pub fn run<H: Handle>(&mut self, handler: &mut H) {
        self.run = true;
        let timeout = time::Duration::from_millis(POLLING_DELAY_MS);
        let desc = H::description();
        while self.run {
            match self.rx.recv_timeout(timeout) {
                Ok(Some(Envelope::Shutdown)) => {
                    debug!("received a shutdown message in {}", desc);
                    self.shutdown();
                },
                Ok(Some(Envelope::Cmd(cmd))) => {
                    trace!("received a command {:?} for {}", cmd, desc);
                    handler.command(cmd);
                },
                Ok(None) => {
                    trace!("command reception timed out, polling new data in {}", desc);
                    handler.poll();
                }
                Err(e) => {
                    error!("unexpected error while receiving messages for {}: {:?}", desc, e);
                },
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

pub struct WorkerStub {
    sender: NotifySender<Envelope>,
    child: thread::JoinHandle<()>,
    description: String,
}

impl WorkerStub {
    pub fn consumer(&self) -> Consumer {
        Consumer {
            sender: self.sender.clone(),
        }
    }

    pub fn shutdown(self) {
        debug!("sending shutdown message to worker for {}", self.description);
        if let Err(e) = self.sender.send(Envelope::Shutdown) {
            error!("unexpected error while shutting down domain worker: {:?}", e);
        }
        debug!("waiting termination of worker for {}", self.description);
        if let Err(e) = self.child.join() {
            error!("unexpected error while joining to domain worker thread: {:?}", e);
        }
    }
}

#[derive(Clone)]
pub struct Consumer {
    sender: NotifySender<Envelope>,
}

impl Consume for Consumer {
    type Item = Command;
    type Error = NotifyError;
    fn consume(&mut self, cmd: Command) -> Result<(), NotifyError> {
        self.sender.send(Envelope::Cmd(cmd))
    }
}

pub fn spawn_worker<H: Handle>() -> WorkerStub {
    let worker = Worker::new();
    let sender = worker.sender();
    let child = thread::spawn(move || {
        let mut handler = H::new();
        let mut worker = worker;
        info!("initiating worker for {}", H::description());
        worker.run(&mut handler);
        info!("terminating worker for {}", H::description());
    });
    WorkerStub {
        sender: sender,
        child: child,
        description: H::description(),
    }
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
            let mut handler = MockHandle {
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
            let mut handler = MockHandle {
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
            let mut handler = MockHandle {
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

    struct MockHandle {
        pollings: mpsc::Sender<()>,
        commands: mpsc::Sender<Command>,
    }

    impl Handle for MockHandle {
        fn new() -> MockHandle { unimplemented!() }
        fn description() -> String { "mock domain handler".to_string() }
        fn command(&mut self, cmd: Command) {
            self.commands.send(cmd).unwrap();
        }

        fn poll(&mut self) {
            self.pollings.send(()).unwrap();
        }
    }
}

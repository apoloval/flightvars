//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::sync;
use std::sync::mpsc;
use std::time;

pub fn notification<T: Send>() -> (NotifySender<T>, NotifyReceiver<T>) {
    let (tx, rx) = mpsc::channel();
    let sleep = sync::Arc::new(Sleep::new());
    let sender = NotifySender {
        tx: tx,
        sleep: sleep.clone(),
    };
    let receiver = NotifyReceiver {
        rx: rx,
        sleep: sleep,
    };
    (sender, receiver)
}

pub struct NotifySender<T: Send> {
    tx: mpsc::Sender<T>,
    sleep: sync::Arc<Sleep>,
}

impl<T: Send> NotifySender<T> {
    pub fn send(&self, value: T) -> NotifyResult<()> {
        try!(self.tx.send(value));
        self.sleep.awake();
        Ok(())
    }
}

impl<T: Send> Clone for NotifySender<T> {
    fn clone(&self) -> NotifySender<T> {
        NotifySender {
            tx: self.tx.clone(),
            sleep: self.sleep.clone(),
        }
    }
}

pub struct NotifyReceiver<T: Send> {
    rx: mpsc::Receiver<T>,
    sleep: sync::Arc<Sleep>,
}

impl<T: Send> NotifyReceiver<T> {
    pub fn recv_timeout(&self, timeout: time::Duration) -> NotifyResult<Option<T>> {
        let result = self.recv();
        if let Ok(None) = result {
            self.sleep.wait(timeout);
            self.recv()
        } else {
            result
        }
    }

    pub fn recv(&self) -> NotifyResult<Option<T>> {
        match self.rx.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(mpsc::TryRecvError::Disconnected) => Err(NotifyError),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
        }
    }
}

pub struct Sleep {
    mutex: sync::Mutex<()>,
    condition: sync::Condvar,
}

impl Sleep {
    pub fn new() -> Sleep {
        Sleep {
            mutex: sync::Mutex::new(()),
            condition: sync::Condvar::new(),
        }
    }

    pub fn awake(&self) {
        self.condition.notify_all();
    }

    pub fn wait(&self, timeout: time::Duration) -> sync::WaitTimeoutResult {
        // We use unwrap here since errors are just possible in case of mutex poisoning.
        // Since we control the actions over the mutex, we can ensure this never happens.
        let lock = self.mutex.lock().unwrap();
        let (_, result) = self.condition.wait_timeout(lock, timeout).unwrap();
        result
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct NotifyError;

impl<T> From<mpsc::SendError<T>> for NotifyError {
    fn from(_: mpsc::SendError<T>) -> NotifyError {
        NotifyError
    }
}

pub type NotifyResult<T> = Result<T, NotifyError>;

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::thread;
    use std::time;

    use super::*;

    #[test]
    fn should_send_and_receive() {
        let (tx, rx) = notification();
        let (echo_tx, echo_rx) = mpsc::channel();
        let child = thread::spawn(move || {
            let msg = rx.recv_timeout(time::Duration::new(1, 0)).unwrap();
            echo_tx.send(msg).unwrap();
        });
        assert!(tx.send(42).is_ok());
        assert_eq!(echo_rx.recv().unwrap(), Some(42));
        assert!(child.join().is_ok());
    }

    #[test]
    fn should_timeout_if_no_msg_was_received() {
        let (_tx, rx) = notification::<isize>();
        let (echo_tx, echo_rx) = mpsc::channel();
        let child = thread::spawn(move || {
            let msg = rx.recv_timeout(time::Duration::from_millis(50)).unwrap();
            echo_tx.send(msg).unwrap();
        });
        assert_eq!(echo_rx.recv().unwrap(), None);
        assert!(child.join().is_ok());
    }

    #[test]
    fn should_fail_when_producer_is_closed() {
        let (echo_tx, echo_rx) = mpsc::channel();
        let child;
        {
            let (_tx, rx) = notification::<isize>();
            child = thread::spawn(move || {
                let result = rx.recv_timeout(time::Duration::from_millis(50));
                echo_tx.send(result).unwrap();
            });
        }
        assert_eq!(echo_rx.recv().unwrap().unwrap_err(), NotifyError);
        assert!(child.join().is_ok());
    }

    #[test]
    fn should_fail_when_consumer_is_closed() {
        let (tx, _) = notification();
        assert_eq!(tx.send(42).unwrap_err(), NotifyError);
    }
}

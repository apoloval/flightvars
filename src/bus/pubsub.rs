//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::result;
use std::sync::mpsc;

pub enum Error {
    Unavailable
}

impl<T> From<mpsc::SendError<T>> for Error {
    fn from(_: mpsc::SendError<T>) -> Error { Error::Unavailable }
}

pub type Result<T> = result::Result<T, Error>;


pub trait Publish<T> {
    fn publish(&mut self, value: T) -> Result<()>;
}

impl<T, M: From<T>> Publish<T> for mpsc::Sender<M> {
    fn publish(&mut self, value: T) -> Result<()> {
        try!(self.send(From::from(value)));
        Ok(())
    }
}

pub trait Subscribe<T, P: Publish<T>> {
    fn subscribe(&mut self, subs: P) -> Result<()>;
}

impl<T, C: Publish<T>, M: From<C>> Subscribe<T, C> for mpsc::Sender<M> {
    fn subscribe(&mut self, subs: C) -> Result<()> {
        try!(self.send(From::from(subs)));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::thread;

    use super::*;

    #[derive(Debug, PartialEq)]
    enum ConsumerMsg {
        Pub(i32)
    }

    impl From<i32> for ConsumerMsg {
        fn from(n: i32) -> ConsumerMsg { ConsumerMsg::Pub(n) }
    }

    #[derive(Debug, PartialEq)]
    struct FakeConsumer;

    impl Publish<i32> for FakeConsumer {
        fn publish(&mut self, _: i32) -> Result<()> { unimplemented!() }
    }

    #[derive(Debug, PartialEq)]
    enum SubscriberMsg {
        Sub(FakeConsumer)
    }

    impl From<FakeConsumer> for SubscriberMsg {
        fn from(c: FakeConsumer) -> SubscriberMsg { SubscriberMsg::Sub(c) }
    }


    #[test]
    fn should_wrap_sender_with_publish() {
        let (mut tx, rx) = mpsc::channel::<ConsumerMsg>();
        let child = thread::spawn(move || {
            assert_eq!(rx.recv().unwrap(), ConsumerMsg::Pub(42));
        });
        let p: &mut Publish<i32> = &mut tx;
        p.publish(42).ok().unwrap();
        child.join().unwrap();
    }

    #[test]
    fn should_wrap_sender_with_subscribe() {
        let (mut tx, rx) = mpsc::channel::<SubscriberMsg>();
        let child = thread::spawn(move || {
            assert_eq!(rx.recv().unwrap(), SubscriberMsg::Sub(FakeConsumer));
        });
        let p: &mut Subscribe<i32, FakeConsumer> = &mut tx;
        p.subscribe(FakeConsumer).ok().unwrap();
        child.join().unwrap();
    }
}

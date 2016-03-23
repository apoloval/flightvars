//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::marker::PhantomData;
use std::sync::mpsc;

use mio;

pub trait Consume {
    type Item;
    type Error;
    fn consume(&mut self, value: Self::Item) -> Result<(), Self::Error>;

    fn map<T, F>(self, f: F) -> ConsumeMap<T, Self, F>
    where F: FnMut(T) -> Self::Item, Self: Sized {
        ConsumeMap {
            _phantom: PhantomData,
            consumer: self,
            mapping: f
        }
    }
}

impl<T> Consume for mpsc::Sender<T> {
    type Item = T;
    type Error = io::Error;

    fn consume(&mut self, value: T) -> Result<(), Self::Error> {
        self.send(value).map_err(|_| {
            io::Error::new(io::ErrorKind::BrokenPipe, "cannot write to mpsc::Sender")
        })
    }
}


impl<T> Consume for mio::Sender<T> where T: Send {
    type Item = T;
    type Error = io::Error;

    fn consume(&mut self, value: T) -> Result<(), Self::Error> {
        self.send(value).map_err(|_| {
            io::Error::new(io::ErrorKind::BrokenPipe, "cannot write to mio::Sender")
        })
    }
}


pub struct ConsumeMap<T, C, F> where C: Consume, F: FnMut(T) -> C::Item {
    _phantom: PhantomData<T>,
    consumer: C,
    mapping: F
}

impl<T, C, F> Consume for ConsumeMap<T, C, F> where C: Consume, F: FnMut(T) -> C::Item {
    type Item = T;
    type Error = C::Error;

    fn consume(&mut self, value: T) -> Result<(), Self::Error> {
        let mapped = (self.mapping)(value);
        self.consumer.consume(mapped)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn should_map_consumer() {
        let (tx, rx) = mpsc::channel::<usize>();
        let mut mapped = tx.map(|s: &str| s.len());
        mapped.consume("foobar");
        assert_eq!(rx.recv().unwrap(), 6);
    }
}

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

pub trait Consume {
    type Item;
    type Error;
    fn consume(&mut self, value: Self::Item) -> Result<(), Self::Error>;    
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

#[derive(Clone)]
pub struct SinkConsumer<T> {
    _phantom: PhantomData<T>
}

impl<T> Consume for SinkConsumer<T> {
    type Item = T;
    type Error = ();

    fn consume(&mut self, _: T) -> Result<(), Self::Error> { Ok(()) }
}

#[cfg(test)]
pub fn sink_consumer<T>() -> SinkConsumer<T> {
    SinkConsumer { _phantom: PhantomData }
}


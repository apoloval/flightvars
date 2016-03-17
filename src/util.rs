//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::sync::mpsc;

pub trait Consume<T> {
    type Error;
    fn consume(&mut self, value: T) -> Result<(), Self::Error>;
}

impl<T> Consume<T> for mpsc::Sender<T> {
    type Error = mpsc::SendError<T>;

    fn consume(&mut self, value: T) -> Result<(), Self::Error> {
        self.send(value)
    }
}

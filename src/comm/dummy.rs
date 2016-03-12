//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io;
use std::sync::mpsc;

use comm::*;

pub enum StreamEvent<M> {
    Message(M),
    Shutdown
}

pub enum ListenerEvent<M> {
    Accept((StreamEventChannel<M>, StreamEventChannel<M>)),
    Shutdown
}

pub struct Channel<T> {
    tx: mpsc::Sender<T>,
    rx: mpsc::Receiver<T>
}

pub type StreamEventChannel<M> = Channel<StreamEvent<M>>;
pub type ListenerEventChannel<M> = Channel<ListenerEvent<M>>;

impl<M> ShutdownInterruption for Channel<M> where mpsc::Sender<M>: Interrupt {
    type Int = mpsc::Sender<M>;

    fn shutdown_interruption(&mut self) -> mpsc::Sender<M> {
        self.tx.clone()
    }
}

impl<M> Interrupt for mpsc::Sender<StreamEvent<M>> {
    fn interrupt(self) { self.send(StreamEvent::Shutdown).unwrap(); }
}

impl<M> Interrupt for mpsc::Sender<ListenerEvent<M>> {
    fn interrupt(self) { self.send(ListenerEvent::Shutdown).unwrap(); }
}

pub struct DummyTransport<M> {
    listener: ListenerEventChannel<M>
}

impl<M> Transport for DummyTransport<M> {
    type Input = StreamEventChannel<M>;
    type Output = StreamEventChannel<M>;
    type Listener = ListenerEventChannel<M>;

    fn listener(&mut self) -> &mut Self::Listener {
        &mut self.listener
    }
}

impl<M> Listen<StreamEventChannel<M>, StreamEventChannel<M>> for ListenerEventChannel<M> {
    fn listen(&mut self) -> io::Result<(StreamEventChannel<M>, StreamEventChannel<M>)> {
        match self.rx.recv().unwrap() {
            ListenerEvent::Accept((i, o)) => Ok((i, o)),
            ListenerEvent::Shutdown => Err(io::Error::new(
                io::ErrorKind::Interrupted, "listener interrupted")),
        }
    }
}

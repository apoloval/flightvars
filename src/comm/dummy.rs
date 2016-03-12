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

pub type StreamEventSender<M> = mpsc::Sender<StreamEvent<M>>;

impl<M> Interrupt for StreamEventSender<M> {
    fn interrupt(self) { self.send(StreamEvent::Shutdown).unwrap() }
}

pub type StreamEventReceiver<M> = mpsc::Receiver<StreamEvent<M>>;

pub enum ListenerEvent<M> {
    Accept((DummyTransportInput<M>, DummyTransportOutput<M>)),
    Shutdown
}

#[derive(Clone)]
pub struct ListenerEventSender<M>(mpsc::Sender<ListenerEvent<M>>);

impl<M> ListenerEventSender<M> {
    pub fn new_connection(&self) -> (StreamEventSender<M>, StreamEventReceiver<M>) {
        let (input, in_tx) = DummyTransportInput::new();
        let (output, out_rx) = DummyTransportOutput::new();
        self.0.send(ListenerEvent::Accept((input, output))).unwrap();
        (in_tx, out_rx)
    }
}

impl<M> Interrupt for ListenerEventSender<M> {
    fn interrupt(self) {
        self.0.send(ListenerEvent::Shutdown).unwrap();
    }
}

pub type ListenerEventReceiver<M> = mpsc::Receiver<ListenerEvent<M>>;

pub type ListenerInterruptor<M> = mpsc::Sender<ListenerEvent<M>>;

impl<M> Interrupt for ListenerInterruptor<M> {
    fn interrupt(self) { self.send(ListenerEvent::Shutdown).unwrap(); }
}

pub struct DummyTransportInput<M> {
    tx: StreamEventSender<M>,
    rx: StreamEventReceiver<M>
}

impl<M> DummyTransportInput<M> {
    pub fn new() -> (DummyTransportInput<M>, StreamEventSender<M>) {
        let (tx, rx) = mpsc::channel();
        let tx_out = tx.clone();
        let input = DummyTransportInput { tx: tx, rx: rx };
        (input, tx_out)
    }

    pub fn recv(&self) -> StreamEvent<M> { self.rx.recv().unwrap() }
}

impl<M> ShutdownInterruption for DummyTransportInput<M> {
    type Int = StreamEventSender<M>;
    fn shutdown_interruption(&mut self) -> StreamEventSender<M> { self.tx.clone() }
}

pub struct DummyTransportOutput<M> {
    tx: StreamEventSender<M>
}

impl<M> DummyTransportOutput<M> {
    pub fn new() -> (DummyTransportOutput<M>, StreamEventReceiver<M>) {
        let (tx, rx) = mpsc::channel();
        let output = DummyTransportOutput { tx: tx };
        (output, rx)
    }

    pub fn send(&self, msg: M) {
        self.tx.send(StreamEvent::Message(msg)).unwrap()
    }
}

pub struct DummyTransportListener<M> {
    tx: ListenerEventSender<M>,
    rx: ListenerEventReceiver<M>
}

impl<M> DummyTransportListener<M> {
    pub fn new() -> DummyTransportListener<M> {
        let (tx, rx) = mpsc::channel();
        let listener = DummyTransportListener {
            tx: ListenerEventSender(tx),
            rx: rx };
        listener
    }
}

impl<M> Listen<DummyTransportInput<M>, DummyTransportOutput<M>> for DummyTransportListener<M> {
    fn listen(&mut self) -> io::Result<(DummyTransportInput<M>, DummyTransportOutput<M>)> {
        match self.rx.recv().unwrap() {
            ListenerEvent::Accept((i, o)) => Ok((i, o)),
            ListenerEvent::Shutdown => Err(io::Error::new(
                io::ErrorKind::Interrupted, "listener interrupted")),
        }
    }
}

impl<M> ShutdownInterruption for DummyTransportListener<M> {
    type Int = ListenerEventSender<M>;
    fn shutdown_interruption(&mut self) -> ListenerEventSender<M> {
        ListenerEventSender(self.tx.0.clone())
    }
}

pub struct DummyTransport<M> {
    listener: DummyTransportListener<M>
}

impl<M> DummyTransport<M> {
    pub fn new(listener: DummyTransportListener<M>) -> DummyTransport<M> {
        DummyTransport { listener: listener }
    }
}

impl<M> Transport for DummyTransport<M> {
    type Input = DummyTransportInput<M>;
    type Output = DummyTransportOutput<M>;
    type Listener = DummyTransportListener<M>;

    fn listener(&mut self) -> &mut Self::Listener {
        &mut self.listener
    }
}

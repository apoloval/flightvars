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

pub enum StreamEvent<T> {
    Message(T),
    Shutdown
}

pub struct StreamEventSender<T>(mpsc::Sender<StreamEvent<T>>);

impl<T> StreamEventSender<T> {
    pub fn send(&self, msg: T) { self.0.send(StreamEvent::Message(msg)).unwrap() }
}

impl<T> Interrupt for StreamEventSender<T> {
    fn interrupt(self) { self.0.send(StreamEvent::Shutdown).unwrap() }
}

pub struct StreamEventReceiver<T>(mpsc::Receiver<StreamEvent<T>>);

pub struct MessageSender<T>(mpsc::Sender<T>);

pub struct MessageReceiver<T>(mpsc::Receiver<T>);

impl<T> MessageReceiver<T> {
    pub fn recv(&self) -> T { self.0.recv().unwrap() }
}

pub enum ListenerEvent<I, O> {
    Accept((DummyTransportInput<I>, DummyTransportOutput<O>)),
    Shutdown
}

#[derive(Clone)]
pub struct ListenerEventSender<I, O>(mpsc::Sender<ListenerEvent<I, O>>);

impl<I, O> ListenerEventSender<I, O> {
    pub fn new_connection(&self) -> (StreamEventSender<I>, MessageReceiver<O>) {
        let (input, in_tx) = DummyTransportInput::new();
        let (output, out_rx) = DummyTransportOutput::new();
        self.0.send(ListenerEvent::Accept((input, output))).unwrap();
        (in_tx, out_rx)
    }
}

impl<I, O> Interrupt for ListenerEventSender<I, O> {
    fn interrupt(self) {
        self.0.send(ListenerEvent::Shutdown).unwrap();
    }
}

pub type ListenerEventReceiver<I, O> = mpsc::Receiver<ListenerEvent<I, O>>;

pub type ListenerInterruptor<I, O> = mpsc::Sender<ListenerEvent<I, O>>;

impl<I, O> Interrupt for ListenerInterruptor<I, O> {
    fn interrupt(self) { self.send(ListenerEvent::Shutdown).unwrap(); }
}

pub struct DummyTransportInput<T> {
    tx: StreamEventSender<T>,
    rx: StreamEventReceiver<T>
}

impl<T> DummyTransportInput<T> {
    pub fn new() -> (DummyTransportInput<T>, StreamEventSender<T>) {
        let (tx, rx) = mpsc::channel();
        let tx_out = StreamEventSender(tx.clone());
        let input = DummyTransportInput {
            tx: StreamEventSender(tx),
            rx: StreamEventReceiver(rx)
        };
        (input, tx_out)
    }

    pub fn recv(&self) -> StreamEvent<T> { self.rx.0.recv().unwrap() }
}

impl<T> ShutdownInterruption for DummyTransportInput<T> {
    type Int = StreamEventSender<T>;
    fn shutdown_interruption(&mut self) -> StreamEventSender<T> {
        StreamEventSender(self.tx.0.clone())
    }
}

impl<T> Identify for DummyTransportInput<T> {
    fn id(&self) -> String {
        let ptr = self as *const Self as usize;
        format!("dummy transport #{}", ptr)
    }
}

pub struct DummyTransportOutput<T> {
    tx: MessageSender<T>
}

impl<T> DummyTransportOutput<T> {
    pub fn new() -> (DummyTransportOutput<T>, MessageReceiver<T>) {
        let (tx, rx) = mpsc::channel();
        let output = DummyTransportOutput { tx: MessageSender(tx) };
        (output, MessageReceiver(rx))
    }

    pub fn send(&self, msg: T) {
        self.tx.0.send(msg);
    }
}

pub struct DummyTransportListener<I, O> {
    tx: ListenerEventSender<I, O>,
    rx: ListenerEventReceiver<I, O>
}

impl<I, O> DummyTransportListener<I, O> {
    pub fn new() -> DummyTransportListener<I, O> {
        let (tx, rx) = mpsc::channel();
        let listener = DummyTransportListener {
            tx: ListenerEventSender(tx),
            rx: rx };
        listener
    }
}

impl<I, O> Listen<DummyTransportInput<I>, DummyTransportOutput<O>> for DummyTransportListener<I, O> {
    fn listen(&mut self) -> io::Result<(DummyTransportInput<I>, DummyTransportOutput<O>)> {
        match self.rx.recv().unwrap() {
            ListenerEvent::Accept((i, o)) => Ok((i, o)),
            ListenerEvent::Shutdown => Err(io::Error::new(
                io::ErrorKind::Interrupted, "listener interrupted")),
        }
    }
}

impl<I, O> ShutdownInterruption for DummyTransportListener<I, O> {
    type Int = ListenerEventSender<I, O>;
    fn shutdown_interruption(&mut self) -> ListenerEventSender<I, O> {
        ListenerEventSender(self.tx.0.clone())
    }
}

pub struct DummyTransport<I, O> {
    listener: DummyTransportListener<I, O>
}

impl<I, O> DummyTransport<I, O> {
    pub fn new(listener: DummyTransportListener<I, O>) -> DummyTransport<I, O> {
        DummyTransport { listener: listener }
    }
}

impl<I, O> Transport for DummyTransport<I, O> {
    type Input = DummyTransportInput<I>;
    type Output = DummyTransportOutput<O>;
    type Listener = DummyTransportListener<I, O>;

    fn listener(&mut self) -> &mut Self::Listener {
        &mut self.listener
    }
}

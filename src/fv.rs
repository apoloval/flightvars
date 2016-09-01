//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::boxed::Box;
use std::ffi::OsStr;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use config::*;
use domain;
use domain::DomainDispatcher;
use io::*;
use proto::*;

pub struct FlightVars {
    cmd_channel: mpsc::Receiver<FlightVarsCommand>,
    domains: DomainDispatcher,
    iocp: CompletionPort<Box<Protocol>>,
    stop: bool,
}

unsafe impl Send for FlightVars {}

impl FlightVars {
    
    pub fn new(settings: &OacspSerialSettings) -> io::Result<FlightVarsHandler> {
        let domains = try!(DomainDispatcher::new());
        let iocp = try!(CompletionPort::new());
        let (tx, rx) = mpsc::channel();
        let mut fv = FlightVars { 
            cmd_channel: rx, 
            domains: domains, 
            iocp: iocp,
            stop: false,
        };
        fv.open_serial_ports(settings);
        let join_handle = thread::spawn(move || fv.run());
        let handler = FlightVarsHandler {
            join_handle: join_handle,
            cmd_channel: tx,
        };
        Ok(handler)
    }
    
    fn open_serial_ports(&mut self, settings: &OacspSerialSettings) {
        // TODO: break coupleness among serial ports, OACSP and fixed baud-rate
        for port in &settings.ports {
            match self.open_serial_port(port) {
                Ok(_) => {
                    info!("serial port {:?} successfully configured", port);
                }
                Err(e) => {
                    error!("cannot configure serial port {:?}: {:?}", port, e);
                }
            }
        }
    }
    
    fn open_serial_port(&mut self, port: &OsStr) -> io::Result<()> {
        let baudrate = 9600;
        debug!("opening serial port {:?} at {} bauds", port, baudrate);
        let mut serial = try!(Serial::open_arduino(port.to_str().unwrap(), baudrate));
        debug!("setting read-upon-available timeouts for serial port {:?}", port);
        try!(serial.set_timeouts(&SerialTimeouts::ReadUponAvailable));
        debug!("initializing OACSP protocol for serial port {:?}", port);
        let proto = Oacsp::new(serial.as_device(), self.domains.clone());
        debug!("attaching serial port {:?} to IOCP port", port);
		try!(self.iocp.attach(Box::new(proto)));
		Ok(())
    } 
    
    fn run(&mut self) {
        self.stop = false;
        while !self.stop {
            self.process_io_event();
            self.process_domain_events();
            self.process_commands();
        }
    }
    
    fn process_commands(&mut self) {
        loop {
            match self.cmd_channel.try_recv() {
                Ok(FlightVarsCommand::Close) => { self.stop = true; }
                _ => { return; },
            }
        }
    }
    
    fn process_io_event(&mut self) {
        match self.iocp.process_event(&Duration::from_millis(50)) {
            _ => {},
        }
    }
    
    fn process_domain_events(&mut self) {
        let mut events = Vec::new();
        if let Err(e) = self.domains.with_all_domains(|domain| domain.poll(&mut events)) {
            error!("unexpected IO error while polling domain events: {:?}", e);
        }
        for ev in events {
            self.send_domain_event(ev);
        }
    }
    
    fn send_domain_event(&mut self, ev: domain::Event) {
        match self.iocp.handler(&ev.device) {
            Some(handler) => {
                if let Err(e) = handler.send_update(&ev.domain, ev.variable, ev.value) {
                    error!("cannot send domain event to device {}: {:?}", ev.device, e);
                }
            }
            None => {
                error!("cannot find a handler for device {} while sending domain event", ev.device);
            }
        }
    }
}

pub enum FlightVarsCommand {
    Close
}

pub struct FlightVarsHandler {
    join_handle: thread::JoinHandle<()>,
    cmd_channel: mpsc::Sender<FlightVarsCommand>   
}

impl FlightVarsHandler {
    
    pub fn close(self) {
        self.cmd_channel.send(FlightVarsCommand::Close).unwrap();
        self.join_handle.join().unwrap();
    }
}

//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ffi::OsString;
use std::fmt;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serial;
use serial::SerialPort;

use comm::{Interrupt, Listen};


#[derive(Debug, Eq, PartialEq)]
enum PortStatus {
    Available, InUse,
}

struct PortInfo {
    name: SerialAddr,
    status: PortStatus,
}

pub struct PortScanner {
    ports: Vec<PortInfo>,
    tx: PortScannerSender,
    rx: PortScannerReceiver,
}

impl PortScanner {
    pub fn new() -> io::Result<PortScanner> {
        let mut ports = Vec::with_capacity(30);
        for i in 3..4 {
            let port_name = format!("COM{}", i);
            let port_info = PortInfo {
                name: SerialAddr(OsString::from(port_name)),
                status: PortStatus::Available,
            };
            ports.push(port_info);
        }
        let (tx, rx) = mpsc::channel();
        Ok(PortScanner { ports: ports,  tx: tx, rx: rx })
    }
}

impl Listen for PortScanner {
    type ConnAddr = SerialAddr;
    type Input = serial::SystemPort;
    type Output = serial::SystemPort;
    type Int = PortScannerSender;
    
    fn listen(&mut self) -> io::Result<(serial::SystemPort, serial::SystemPort, SerialAddr)> {
        loop {
            if let Ok(PortScannerMsg::Shutdown) = self.rx.try_recv() { 
            	return Err(io::Error::new(
            	        io::ErrorKind::ConnectionAborted, 
            	        "scanner received a shutdown message"));
            }
            for p in self.ports.iter_mut() {
                if p.status == PortStatus::Available {
                    debug!("trying to open serial port {}", p.name);
                    match serial::open(&p.name.0) {
                        Ok(mut open_port) => {
                            info!("serial port {} successfully open", p.name);
                            
                            let mut settings = serial::PortSettings::default();
                            settings.baud_rate = serial::BaudRate::Baud9600;
                            try!(open_port.configure(&settings));
                            try!(open_port.set_rts(true));
                            try!(open_port.set_dtr(true));
                            
                            p.status = PortStatus::InUse;
                            let read_port = try!(open_port.try_clone());
                            let write_port = open_port;
                            return Ok((read_port, write_port, p.name.clone()));
                        },
                        Err(e) => {
                        	debug!("cannot open serial port {}: {:?}", p.name, e);
                        },
                    }
                }
            }
            thread::sleep(Duration::from_millis(1000));
        };
    }

    fn shutdown_interruption(&mut self) -> PortScannerSender {
        self.tx.clone()
    }
}

#[derive(Clone)]
pub struct SerialAddr(OsString);

impl fmt::Display for SerialAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}


pub enum PortScannerMsg {
    Shutdown
}

pub type PortScannerSender = mpsc::Sender<PortScannerMsg>;
pub type PortScannerReceiver = mpsc::Receiver<PortScannerMsg>;

pub type SerialInterruptor = PortScannerSender;

impl Interrupt for PortScannerSender {
    fn interrupt(self) {
        if let Err(e) = self.send(PortScannerMsg::Shutdown) {
            error!("cannot send shutdown message to serial port scanner: {:?}", e);
        }
    }
}
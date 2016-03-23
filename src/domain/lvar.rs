//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ffi::CString;
use std::io;
use std::thread;

use libc::c_char;
use mio;

use domain::types::*;
use util::Consume;

const POLLING_DELAY_MS: u64 = 50;

#[derive(Debug)]
pub enum Envelope {
    Cmd(Command),
    Shutdown
}

pub struct Domain {
    worker: thread::JoinHandle<()>,
    tx: mio::Sender<Envelope>
}

impl Domain {
    pub fn new() -> Domain {
        let (worker, tx) = spawn_worker();
        Domain { worker: worker, tx: tx }
    }

    pub fn shutdown(self) {
        self.tx.send(Envelope::Shutdown).unwrap_or_else(|e| {
            warn!("unexpected error while sending shutdown message to lvar domain: {}", e);
        });
        self.worker.join().unwrap_or_else(|_| {
            warn!("unexpected error while waiting for lvar domain worker thread");
        });
    }

    pub fn consumer(&self) -> Consumer {
        Consumer { tx: self.tx.clone() }
    }
}

#[derive(Clone)]
pub struct Consumer {
    tx: mio::Sender<Envelope>
}

impl Consume for Consumer {
    type Item = Command;
    type Error = mio::NotifyError<Envelope>;
    fn consume(&mut self, cmd: Command) -> Result<(), mio::NotifyError<Envelope>> {
        self.tx.send(Envelope::Cmd(cmd))
    }
}


struct Observer {
    lvar: String,
    client: Client,
    retain: Option<Value>,
}

impl Observer {
}

struct Context {
    observers: Vec<Observer>,
}

impl Context {
    pub fn new()  -> Context {
        Context {
            observers: Vec::new(),
        }
    }

    fn process_write(&mut self, lvar: &str, value: Value) {
        debug!("writing value {} to lvar {}", value, lvar);
        // TODO: implement this
    }

    fn process_obs(&mut self, lvar: &str, client: Client) {
        debug!("client {} observing lvar {}", client.name(), lvar);
        self.observers.push(Observer {
            lvar: lvar.to_string(),
            client: client,
            retain: None,
        });
    }
}

fn spawn_worker() -> (thread::JoinHandle<()>, mio::Sender<Envelope>) {
    let event_loop = mio::EventLoop::new().unwrap();
    let tx = event_loop.channel();
    let worker = thread::spawn(move || {
        let mut event_loop = event_loop;
        let mut ctx = Context::new();
        event_loop.timeout_ms((), POLLING_DELAY_MS).unwrap();
        event_loop.run(&mut ctx).unwrap();
    });
    (worker, tx)
}

impl mio::Handler for Context {
    type Timeout = ();
    type Message = Envelope;

    fn timeout(&mut self, event_loop: &mut mio::EventLoop<Context>, _: ()) {
        // TODO: implement this
    }

    fn notify(&mut self, event_loop: &mut mio::EventLoop<Context>, msg: Envelope) {
        match msg {
            Envelope::Cmd(Command::Write(Var::LVar(lvar), value)) => {
                self.process_write(&lvar, value);
            },
            Envelope::Cmd(Command::Observe(Var::LVar(lvar), client)) => {
                self.process_obs(&lvar, client);
            },
            Envelope::Shutdown => event_loop.shutdown(),
            other => {
                warn!("LVAR domain received an unexpected message: {:?}", other);
            },
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////
// The following code provides the low level routines to access FSX/P3D panels. //
//////////////////////////////////////////////////////////////////////////////////

pub type Bool = i32;
pub type Enum = i32;
pub type Flags32 = u32;
pub type GeneratePhase = u32;
pub type Id = i32;

pub fn check_named_variable(name: &str) -> Id {
    unsafe {
        let func = (*Panels).check_named_variable;
        let name = CString::new(name).unwrap();
        (func)(name.as_ptr())
    }
}

pub fn get_named_variable_value(id: Id) -> f64 {
    unsafe {
        let func = (*Panels).get_named_variable_value;
        (func)(id)
    }
}

pub fn is_initialized() -> bool {
    unsafe { !Panels.is_null() }
}

/// This struct declares all panel related functions exported by FSX/P3D.
/// Most of the fields (those whose name starts with underscore) are not used by FlightVars.
/// In order to simplify the code, their declared function signatures don't correspond to the
/// actual ones. If you want to use such functions you'll have to set their signatures to that
/// declared in `struct PANELS` in `gauges.h` header file of P3D SDK.
pub struct PanelFunctions {
    _mod_id: Id,
    _mod_init: extern "stdcall" fn(),
    _mod_deinit: extern "stdcall" fn(),
    _mod_flags: Flags32,
    _mod_priority: u32,
    _mod_version: u32,

    _reserved1: extern "stdcall" fn(),
    _reserved2: extern "stdcall" fn(),
    _reserved3: extern "stdcall" fn(),
    _reserved4: extern "stdcall" fn(),
    _reserved5: extern "stdcall" fn(),
    _is_panel_visible_ident: extern "stdcall" fn(),
    _tooltip_units_getset: extern "stdcall" fn(),
    _reserved7: extern "stdcall" fn(),
    _reserved8: extern "stdcall" fn(),
    _reserved9: extern "stdcall" fn(),
    _reserved10: extern "stdcall" fn(),
    _reserved11: extern "stdcall" fn(),
    _reserved12: extern "stdcall" fn(),
    _reserved13: extern "stdcall" fn(),
    _reserved14: extern "stdcall" fn(),
    _reserved15: extern "stdcall" fn(),
    _reserved16: extern "stdcall" fn(),
    _reserved17: extern "stdcall" fn(),
    _element_list_query: extern "stdcall" fn(),
    _element_list_install: extern "stdcall" fn(),
    _element_list_initialize: extern "stdcall" fn(),
    _element_list_update: extern "stdcall" fn(),
    _element_list_generate: extern "stdcall" fn(),
    _element_list_plot: extern "stdcall" fn(),
    _element_list_erase: extern "stdcall" fn(),
    _element_list_kill: extern "stdcall" fn(),
    _mouse_list_install: extern "stdcall" fn(),
    _mouse_list_register: extern "stdcall" fn(),
    _mouse_list_unregister: extern "stdcall" fn(),
    _panel_window_togle: extern "stdcall" fn(),
    _trigger_key_event: extern "stdcall" fn(),
    _register_var_by_name: extern "stdcall" fn(),
    _initialize_var: extern "stdcall" fn(),
    _initialize_var_by_name: extern "stdcall" fn(),
    _lookup_var: extern "stdcall" fn(),
    _unregister_var_by_name: extern "stdcall" fn(),
    _unregister_all_named_vars: extern "stdcall" fn(),
    _reserved18: extern "stdcall" fn(),
    _reserved19: extern "stdcall" fn(),
    _panel_window_close_ident: extern "stdcall" fn(),
    _panel_window_open_ident: extern "stdcall" fn(),
    _panel_window_toggle_hud_color: extern "stdcall" fn(),
    _panel_window_toggle_hud_units: extern "stdcall" fn(),
    _radio_stack_popup: extern "stdcall" fn(),
    _radio_stack_autoclose: extern "stdcall" fn(),
    pub check_named_variable: extern "stdcall" fn(name: *const c_char) -> Id,
    _register_named_variable: extern "stdcall" fn(),
    pub get_named_variable_value: extern "stdcall" fn(id: Id) -> f64,
    _get_named_variable_typed_value: extern "stdcall" fn(),
    _set_named_variable_value: extern "stdcall" fn(),
    _set_named_variable_typed_value: extern "stdcall" fn(),
    _reserved26: extern "stdcall" fn(),
    _reserved27: extern "stdcall" fn(),
    _get_name_of_named_variable: extern "stdcall" fn(),
    _reserved29: extern "stdcall" fn(),
    _panel_resource_string_get: extern "stdcall" fn(),
    _panel_window_toggle_menu_id: extern "stdcall" fn(),
    _reserved30: extern "stdcall" fn(),
    _reserved31: extern "stdcall" fn(),
    _element_use_color: extern "stdcall" fn(),
    _set_gauge_flags: extern "stdcall" fn(),
    _get_gauge_flags: extern "stdcall" fn(),
    _gauge_calculator_code_precompile: extern "stdcall" fn(),
    _execute_calculator_code: extern "stdcall" fn(),
    _format_calculator_string: extern "stdcall" fn(),
    _reserved32: extern "stdcall" fn(),
    _reserved33: extern "stdcall" fn(),
    _get_units_enum: extern "stdcall" fn(),
    _get_aircraft_var_enum: extern "stdcall" fn(),
    _aircraft_varget: extern "stdcall" fn(),
    _panel_register_c_callback: extern "stdcall" fn(),
    _panel_get_registered_c_callback: extern "stdcall" fn(),
    _panel_get_aircraft_c_callback: extern "stdcall" fn(),
    _send_key_event: extern "stdcall" fn(),
    _register_key_event_handler: extern "stdcall" fn(),
    _unregister_key_event_handler: extern "stdcall" fn(),
    _reserved34: extern "stdcall" fn(),
    _reserved35: extern "stdcall" fn(),
    _process_shared_event_out: extern "stdcall" fn(),
    _is_master: extern "stdcall" fn(),
    _reserved36: extern "stdcall" fn(),
    _set_named_variable_value_sync: extern "stdcall" fn(),
    _set_named_variable_sync_enabled: extern "stdcall" fn(),
}

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut Panels: *mut PanelFunctions = 0 as *mut PanelFunctions;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_init_and_shutdown() {
        let mut domain = Domain::new();
        domain.shutdown();
    }
}

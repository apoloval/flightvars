//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(non_snake_case,non_camel_case_types,non_upper_case_globals,dead_code)]

extern crate libc;

use std::mem;	

use self::libc::{c_char, c_void, c_int, c_ulong, wchar_t};

pub type BYTE = u8;
pub type WORD = u16;
pub type DWORD = c_ulong;
pub type BOOL = c_int;
pub type CHAR = c_char;
pub type WCHAR = wchar_t;

pub type DWORD_PTR = *mut DWORD;
pub type LPDWORD = *mut DWORD;
pub type LPVOID = *mut c_void;
pub type LPCVOID = *const c_void;
pub type ULONG_PTR = DWORD_PTR;
pub type PULONG_PTR = *mut ULONG_PTR;

pub type LPCWSTR = *const WCHAR;
pub type LPWSTR = *mut WCHAR;

pub type HANDLE = *mut LPVOID;
pub type LPHANDLE = *mut HANDLE;

pub const MAXDWORD: DWORD = 0xFFFFFFFF;

pub const INVALID_HANDLE_VALUE: HANDLE = !0 as HANDLE;

pub const INFINITE: DWORD = !0 as DWORD;
pub const ERROR_IO_PENDING: DWORD = 997;

pub const STATUS_WAIT_0: ULONG_PTR 			 	= 0 as ULONG_PTR;
pub const STATUS_ABANDONED_WAIT_0: ULONG_PTR 	= 128 as ULONG_PTR;
pub const STATUS_USER_APC: ULONG_PTR			= 192 as ULONG_PTR;
pub const STATUS_TIMEOUT: ULONG_PTR 			= 258 as ULONG_PTR;
pub const STATUS_PENDING: ULONG_PTR 			= 259 as ULONG_PTR;

pub const GENERIC_READ: DWORD 	 	= 0x80000000;
pub const GENERIC_WRITE: DWORD   	= 0x40000000;
pub const GENERIC_EXECUTE: DWORD 	= 0x20000000;
pub const GENERIC_ALL: DWORD     	= 0x10000000;

pub const CREATE_NEW: DWORD 		= 1;
pub const CREATE_ALWAYS: DWORD 		= 2;
pub const OPEN_EXISTING: DWORD 		= 3;
pub const OPEN_ALWAYS: DWORD 		= 4;
pub const TRUNCATE_EXISTING: DWORD 	= 5;

pub const FILE_ATTRIBUTE_READONLY: DWORD 	= 0x00000001;
pub const FILE_ATTRIBUTE_HIDDEN: DWORD 		= 0x00000002;
pub const FILE_ATTRIBUTE_SYSTEM: DWORD 		= 0x00000004;
pub const FILE_ATTRIBUTE_DIRECTORY: DWORD 	= 0x00000010;
pub const FILE_ATTRIBUTE_ARCHIVE: DWORD 	= 0x00000020;
pub const FILE_ATTRIBUTE_DEVICE: DWORD 		= 0x00000040;
pub const FILE_ATTRIBUTE_NORMAL: DWORD 		= 0x00000080;
pub const FILE_ATTRIBUTE_TEMPORARY: DWORD 	= 0x00000100;

pub const FILE_FLAG_WRITE_THROUGH: DWORD 	= 0x80000000;
pub const FILE_FLAG_NO_BUFFERING: DWORD 	= 0x20000000;
pub const FILE_FLAG_RANDOM_ACCESS: DWORD 	= 0x10000000;
pub const FILE_FLAG_SEQUENTIAL_SCAN: DWORD 	= 0x08000000;
pub const FILE_FLAG_DELETE_ON_CLOSE: DWORD 	= 0x04000000;
pub const FILE_FLAG_OVERLAPPED: DWORD 		= 0x40000000;

pub const fBinary:           DWORD = 0x00000001;
pub const fParity:           DWORD = 0x00000002;
pub const fOutxCtsFlow:      DWORD = 0x00000004;
pub const fOutxDsrFlow:      DWORD = 0x00000008;
pub const fDtrControl:       DWORD = 0x00000010;
pub const fDsrSensitivity:   DWORD = 0x00000040;
pub const fTXContinueOnXoff: DWORD = 0x00000080;
pub const fOutX:             DWORD = 0x00000100;
pub const fInX:              DWORD = 0x00000200;
pub const fErrorChar:        DWORD = 0x00000400;
pub const fNull:             DWORD = 0x00000800;
pub const fRtsControl:       DWORD = 0x00003000;
pub const fAbortOnError:     DWORD = 0x00004000;
pub const fDummy2:           DWORD = 0xFFFF8000;

pub const PURGE_TXABORT: DWORD = 0x0001;
pub const PURGE_RXABORT: DWORD = 0x0002;
pub const PURGE_TXCLEAR: DWORD = 0x0004;
pub const PURGE_RXCLEAR: DWORD = 0x0008;

#[repr(C)]
pub struct COMMTIMEOUTS {
    ReadIntervalTimeout: DWORD,
  	ReadTotalTimeoutMultiplier: DWORD,
  	ReadTotalTimeoutConstant: DWORD,
  	WriteTotalTimeoutMultiplier: DWORD,
  	WriteTotalTimeoutConstant: DWORD,
} 

impl COMMTIMEOUTS {
    pub fn read_upon_available() -> COMMTIMEOUTS {
        COMMTIMEOUTS {
            ReadIntervalTimeout: MAXDWORD,
          	ReadTotalTimeoutMultiplier: MAXDWORD,
          	ReadTotalTimeoutConstant: MAXDWORD - 1,
          	WriteTotalTimeoutMultiplier: 0,
          	WriteTotalTimeoutConstant: 0,
        }
    }
    
    pub fn wait_to_fill() -> COMMTIMEOUTS {
        COMMTIMEOUTS {
            ReadIntervalTimeout: MAXDWORD - 1,
          	ReadTotalTimeoutMultiplier: 0,
          	ReadTotalTimeoutConstant: 0,
          	WriteTotalTimeoutMultiplier: 0,
          	WriteTotalTimeoutConstant: 0,
        }
    }
}

pub type LPCOMMTIMEOUTS = *mut COMMTIMEOUTS;
pub type LPCCOMMTIMEOUTS = *const COMMTIMEOUTS;

#[repr(C)]
pub struct DCB {
    pub DCBlength: DWORD,
  	pub BaudRate: DWORD,
  	pub flags: DWORD,
    pub wReserved: WORD,
    pub XonLim: WORD,
    pub XoffLim: WORD,
    pub ByteSize: BYTE,
    pub Parity: BYTE,
    pub StopBits: BYTE,
    pub XonChar: CHAR,
    pub XoffChar: CHAR,
    pub ErrorChar: CHAR,
    pub EofChar: CHAR,
    pub EvtChar: CHAR,
    pub wReserved1: WORD,
}

pub type LPDCB = *mut DCB;
pub type LPCDCB = *const DCB;

impl DCB {
    pub fn new() -> DCB {
        let mut dcb: DCB = unsafe { mem::zeroed() };
        dcb.DCBlength = mem::size_of_val(&dcb) as DWORD;
        dcb
    }
    
    pub fn setDtrControl(&mut self) {
        self.flags |= fDtrControl;
        self.flags &= !fRtsControl; 
    }
}

#[repr(C)]
pub struct SECURITY_ATTRIBUTES {
    pub nLength: DWORD,
    pub lpSecurityDescriptor: LPVOID,
    pub bInheritHandle: BOOL,
}

pub type LPSECURITY_ATTRIBUTES = *mut SECURITY_ATTRIBUTES;

#[repr(C)]
pub struct OVERLAPPED {
    pub Internal: ULONG_PTR ,
    pub InternalHigh: ULONG_PTR ,
    pub Offset: DWORD,
    pub OffsetHigh: DWORD,
    pub hEvent: HANDLE,
}

impl OVERLAPPED {
    pub fn new() -> OVERLAPPED {
        OVERLAPPED {
            Internal: 0 as ULONG_PTR ,
            InternalHigh: 0 as ULONG_PTR ,
            Offset: 0, 
            OffsetHigh: 0, 
            hEvent: 0 as HANDLE,
        }
    }
}

pub type LPOVERLAPPED = *mut OVERLAPPED;

macro_rules! checked_handle {
    (valid => $func:expr) => ({ 
 		let handle = unsafe { $func };
        if handle == INVALID_HANDLE_VALUE {
            return Err(::std::io::Error::last_os_error());
        }
        handle
    });
    (not_null => $func:expr) => ({ 
 		let handle = unsafe { $func };
        if handle == 0 as HANDLE {
            return Err(::std::io::Error::last_os_error());
        }
        handle
    });
}

macro_rules! checked_result {
    ($func:expr) => ({ 
 		let rc = unsafe { $func };
        if rc == 0 {
            return Err(::std::io::Error::last_os_error());
        }
    });
}

extern "system" {
    
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    
	pub fn CreateFileW(
  		lpFileName: LPCWSTR,
  		dwDesiredAccess: DWORD,
  		dwShareMode: DWORD,
  		lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
   		dwCreationDisposition: DWORD,
  		dwFlagsAndAttributes: DWORD,
  		hTemplateFile: HANDLE) -> HANDLE;
    
    pub fn CreateIoCompletionPort(
      	FileHandle: HANDLE,
      	ExistingCompletionPort: HANDLE,
      	CompletionKey: ULONG_PTR,
      	NumberOfConcurrentThreads: DWORD) -> HANDLE;
    
    pub fn GetCommState(
        hFile: HANDLE,
        lpDCB: LPDCB) -> BOOL;
    
    pub fn GetLastError() -> DWORD;
    
    pub fn GetQueuedCompletionStatus(
		CompletionPort: HANDLE,
     	pNumberOfBytes: LPDWORD,
     	pCompletionKey: PULONG_PTR,
     	ppOverlapped: *mut LPOVERLAPPED,
  	 	dwMilliseconds: DWORD) -> BOOL;
    
    pub fn PurgeComm(
        hFile: HANDLE,
        dwFlags: DWORD) -> BOOL;
    
    pub fn ReadFile(
  		File: HANDLE,
      	pBuffer: LPVOID,
        NumberOfBytesToRead: DWORD,
        pNumberOfBytesRead: LPDWORD,
    	pOverlapped: LPOVERLAPPED) -> BOOL;
    
    pub fn SetCommState(
        hFile: HANDLE,
        lpDCB: LPCDCB) -> BOOL;
    
    pub fn SetCommTimeouts(
        hFile: HANDLE,
        lpCommTimeouts: LPCCOMMTIMEOUTS) -> BOOL;
    
    pub fn WriteFile(
  		hFile: HANDLE,
      	lpBuffer: LPCVOID,
        nNumberOfBytesToWrite: DWORD,
        lpNumberOfBytesWritten: LPDWORD,
    	lpOverlapped: LPOVERLAPPED) -> BOOL;
}
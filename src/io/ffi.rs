//
// FlightVars
// Copyright (c) 2015, 2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(non_snake_case,non_camel_case_types,non_upper_case_globals,dead_code)]

extern crate libc;

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
pub type ULONG_PTR = DWORD_PTR;
pub type PULONG_PTR = *mut ULONG_PTR;

pub type LPCWSTR = *const WCHAR;
pub type LPWSTR = *mut WCHAR;

pub type HANDLE = *mut LPVOID;
pub type LPHANDLE = *mut HANDLE;

pub const INVALID_HANDLE_VALUE: HANDLE = !0 as HANDLE;

pub const INFINITE: DWORD = !0 as DWORD;

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

#[repr(C)]
pub struct SECURITY_ATTRIBUTES {
    pub nLength: DWORD,
    pub lpSecurityDescriptor: LPVOID,
    pub bInheritHandle: BOOL,
}

pub type LPSECURITY_ATTRIBUTES = *mut SECURITY_ATTRIBUTES;

#[repr(C)]
pub struct OVERLAPPED {
    pub Internal: *mut c_ulong,
    pub InternalHigh: *mut c_ulong,
    pub Offset: DWORD,
    pub OffsetHigh: DWORD,
    pub hEvent: HANDLE,
}

impl OVERLAPPED {
    pub fn new() -> OVERLAPPED {
        OVERLAPPED {
            Internal: 0 as *mut c_ulong,
            InternalHigh: 0 as *mut c_ulong,
            Offset: 0, 
            OffsetHigh: 0, 
            hEvent: 0 as HANDLE,
        }
    }
}

pub type LPOVERLAPPED = *mut OVERLAPPED;

extern "system" {
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
    
    pub fn GetQueuedCompletionStatus(
		CompletionPort: HANDLE,
     	pNumberOfBytes: LPDWORD,
     	pCompletionKey: PULONG_PTR,
     	ppOverlapped: *mut LPOVERLAPPED,
  	 	dwMilliseconds: DWORD) -> BOOL;
    
    pub fn ReadFile(
  		File: HANDLE,
      	pBuffer: LPVOID,
        NumberOfBytesToRead: DWORD,
        pNumberOfBytesRead: LPDWORD,
    	pOverlapped: LPOVERLAPPED) -> BOOL;
}
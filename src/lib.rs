//! Simple top level domain (NSSwitch hosts) resolver for a Linux-based
//! development environment.

extern crate libc;

use std::ffi::CStr;
use std::mem::size_of;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::null_mut;
use libc::{
    AF_INET, AF_INET6, hostent, in6_addr, in_addr_t, malloc, strdup,
};

extern "C" {
    fn inet_pton(af: c_int, src: *const c_char, dst: *mut c_void) -> c_int;

    fn inet_addr(cp: *const c_char) -> in_addr_t;
}

/// Using `malloc`, allocate enough memory to hold a single value of type `T`,
/// and populate it with the given `value`.
unsafe fn into_malloc_heap<T>(value: T) -> *mut T {
    let ptr = malloc(size_of::<T>()) as *mut T;
    if !ptr.is_null() {
        *ptr = value;
    }
    ptr
}

const INADDRSZ: c_int = 4;
const IN6ADDRSZ: c_int = 16;

unsafe fn fill_hostent(
    name: *const c_char,
    af: c_int,
    result: *mut hostent,
) {
    // Start by borrowing a reference to `*result`.
    let result = &mut *result;

    result.h_name = strdup(name);
    result.h_aliases = into_malloc_heap(null_mut());
    result.h_addr_list = malloc(size_of::<*mut c_char>() * 2) as *mut *mut c_char;

    match af {
        AF_INET => {
            result.h_addrtype = AF_INET;
            result.h_length = INADDRSZ;
            let addr: in_addr_t = inet_addr(b"127.0.0.1\0" as *const u8 as *const c_char);
            *result.h_addr_list = into_malloc_heap(addr) as *mut c_char;
        }
        AF_INET6 => {
            result.h_addrtype = AF_INET6;
            result.h_length = IN6ADDRSZ;
            let mut addr6: in6_addr = std::mem::uninitialized();
            inet_pton(
                AF_INET6,
                b"::1\0" as *const u8 as *const c_char,
                &mut addr6 as *mut in6_addr as *mut c_void,
            );
            *result.h_addr_list = into_malloc_heap(addr6) as *mut c_char;
        }
        _ => {
            println!("unexpected address family");
        }
    }

    *result.h_addr_list.offset(1) = null_mut();
}

#[repr(C)]
pub enum Status {
    TryAgain = -2,
    Unavailable = -1,
    NotFound = 0,
    Success = 1,
}

pub unsafe extern "C" fn _nss_resolver_gethostbyname2_r(
    name: *const c_char,
    af: c_int,
    result: *mut hostent,
    _buffer: *mut c_char,
    _buflen: usize,
    _errnop: *mut c_int,
    _h_errnop: *mut c_int,
) -> Status {
    // First, convert the C pointer `name` to a Rust string.
    // This fails if the string isn't UTF-8.
    if let Ok(name_str) = CStr::from_ptr(name).to_str() {
        // Find the last dot, if any.
        if let Some(index) = name_str.rfind('.') {
            let name_tld = &name_str[index + 1..];

            let domains = std::env::var("NSS_RESOLVER_TLDS").unwrap_or_else(|_| "test".to_string());
            for domain in domains.split(',') {
                if name_tld.eq_ignore_ascii_case(domain) {
                    fill_hostent(name, af, result);
                    return Status::Success;
                }
            }
        }
    }

    Status::NotFound
}

pub unsafe extern "C" fn _nss_resolver_gethostbyname_r(
    name: *const c_char,
    result: *mut hostent,
    buffer: *mut c_char,
    buflen: usize,
    errnop: *mut c_int,
    h_errnop: *mut c_int,
) -> Status {
    return _nss_resolver_gethostbyname2_r(name, AF_INET, result, buffer, buflen, errnop, h_errnop);
}


pub unsafe extern "C" fn _nss_resolver_gethostbyaddr_r(
    _addr: *const c_void,
    _len: c_int,
    _af: c_int,
    _result: *mut hostent,
    _buffer: *mut c_char,
    _buflen: usize,
    _errnop: *mut c_int,
    _h_errnop: *mut c_int,
) -> Status {
    Status::Unavailable
}

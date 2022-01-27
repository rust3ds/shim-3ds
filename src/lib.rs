#![no_std]

use core::mem::MaybeUninit;
use core::ptr;

extern crate libc;

/// Call this somewhere to force Rust to link this module.
/// The call doesn't need to execute, just exist.
///
/// See https://github.com/rust-lang/rust/issues/47384
pub fn init() {}

extern "C" {
    // Not provided by libc: https://github.com/rust-lang/libc/issues/1995
    fn __errno() -> *mut libc::c_int;
}

#[no_mangle]
extern "C" fn posix_memalign(
    memptr: *mut *mut libc::c_void,
    align: libc::size_t,
    size: libc::size_t,
) -> libc::c_int {
    unsafe {
        *memptr = libc::memalign(align, size);
    }

    0
}

#[no_mangle]
unsafe extern "C" fn realpath(
    path: *const libc::c_char,
    mut resolved_path: *mut libc::c_char,
) -> *mut libc::c_char {
    let path_len = libc::strlen(path);

    if resolved_path.is_null() {
        resolved_path = libc::malloc(path_len + 1) as _;
    }

    libc::strncpy(resolved_path as _, path as _, path_len + 1);

    resolved_path
}

#[no_mangle]
unsafe extern "C" fn clock_gettime(
    clock_id: libc::clockid_t,
    tp: *mut libc::timespec,
) -> libc::c_int {
    let mut retval = -1;
    match clock_id {
        libc::CLOCK_REALTIME => {
            let mut tv = MaybeUninit::uninit();

            retval = libc::gettimeofday(tv.as_mut_ptr(), ptr::null_mut());
            if retval == 0 {
                let tv = tv.assume_init();
                (*tp).tv_nsec = (tv.tv_usec * 1000).into();
                (*tp).tv_sec = tv.tv_sec;
            }
        }
        _ => *__errno() = libc::EINVAL,
    }

    retval
}

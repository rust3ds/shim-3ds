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
                (*tp).tv_nsec = tv.tv_usec * 1000;
                (*tp).tv_sec = tv.tv_sec;
            }
        }
        _ => *__errno() = libc::EINVAL,
    }

    retval
}

#[no_mangle]
unsafe extern "C" fn getrandom(
    buf: *mut libc::c_void,
    mut buflen: libc::size_t,
    flags: libc::c_uint,
) -> libc::ssize_t {
    // TODO: is this needed? Maybe just `buflen = buflen.min(libc::ssize_t::MAX)` ?
    buflen = buflen.min(0x1FFFFFF);

    if flags != 0 {
        // no flags are supported on 3DS
        *__errno() = libc::EINVAL;
        return -1;
    }

    let ret = ctru_sys::PS_GenerateRandomBytes(buf, buflen as libc::c_uint) as libc::ssize_t;
    if ret < 0 {
        // this is kind of a hack, but at least gives some visibility to the
        // error code returned by PS_GenerateRandomBytes I guess? Another option
        // might be to panic, which could use a payload of a specific error type
        // that the ctru panic handler could decode into 3DS-specific human-readable
        // errors.
        *__errno() = ret as libc::c_int;
        -1
    } else {
        // safe because above ensures buflen < isize::MAX
        buflen as libc::ssize_t
    }
}

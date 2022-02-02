#![no_std]

use core::convert::TryFrom;
use core::mem::MaybeUninit;
use core::ptr;

extern crate libc;

// avoid conflicting a real POSIX errno by using a value < 0
// should we define this in ctru-sys somewhere or something?
const ECTRU: libc::c_int = -1;

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
        libc::CLOCK_MONOTONIC => {
            if let Ok(tick) = i64::try_from(ctru_sys::svcGetSystemTick()) {
                retval = 0;

                let sysclock_rate = i64::from(ctru_sys::SYSCLOCK_ARM11);
                (*tp).tv_sec = tick / sysclock_rate;

                // this should always fit in an f64 easily, since it's < sysclock_rate
                let remainder = (tick % sysclock_rate) as f64;

                // cast to i32 rounds toward zero, which should be fine for this use case
                (*tp).tv_nsec = (1000.0 * (remainder / ctru_sys::CPU_TICKS_PER_USEC)) as i32;
            } else {
                // Too many ticks, this device has been on for >1000 years!
                // We would have otherwise given a negative result back to caller
                *__errno() = ECTRU
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
    // Based on https://man7.org/linux/man-pages/man2/getrandom.2.html
    // Technically we only have one source (no true /dev/random), but the
    // behavior should be more expected this way.
    let maxlen = if flags & libc::GRND_RANDOM != 0 {
        512
    } else {
        0x1FFFFFF
    };
    buflen = buflen.min(maxlen);

    let ret = ctru_sys::PS_GenerateRandomBytes(buf, buflen as libc::c_uint);

    // avoid conflicting a real POSIX errno by using a value < 0
    // should we define this in ctru-sys somewhere or something?
    const ECTRU: libc::c_int = -1;

    if ctru_sys::R_SUCCEEDED(ret) {
        // safe because above ensures buflen < isize::MAX
        buflen as libc::ssize_t
    } else {
        // best-effort attempt at translating return codes
        *__errno() = match ctru_sys::R_SUMMARY(ret) as libc::c_uint {
            ctru_sys::RS_WOULDBLOCK => libc::EAGAIN,
            ctru_sys::RS_INVALIDARG | ctru_sys::RS_WRONGARG => {
                match ctru_sys::R_DESCRIPTION(ret) as libc::c_uint {
                    // most likely user error, forgot to initialize PS module
                    ctru_sys::RD_INVALID_HANDLE => ECTRU,
                    _ => libc::EINVAL,
                }
            }
            _ => ECTRU,
        };
        -1
    }
}

#![no_std]

/// Call this somewhere to force Rust to link this module.
/// The call doesn't need to execute, just exist.
///
/// See <https://github.com/rust-lang/rust/issues/47384>
pub fn init() {}

extern "C" {
    // Not provided by libc: https://github.com/rust-lang/libc/issues/1995
    fn __errno() -> *mut libc::c_int;
}

#[no_mangle]
pub unsafe extern "C" fn posix_memalign(
    memptr: *mut *mut libc::c_void,
    align: libc::size_t,
    size: libc::size_t,
) -> libc::c_int {
    *memptr = libc::memalign(align, size);

    0
}

#[no_mangle]
pub unsafe extern "C" fn realpath(
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
pub unsafe extern "C" fn getrandom(
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

    let ret = ctru_sys::PS_GenerateRandomBytes(buf, buflen);

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

#[no_mangle]
pub extern "C" fn sysconf(name: libc::c_int) -> libc::c_long {
    match name {
        libc::_SC_PAGESIZE => 0x1000,
        _ => {
            unsafe { *__errno() = libc::EINVAL };
            -1
        }
    }
}

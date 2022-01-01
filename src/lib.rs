extern crate libc;

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
    resolved_path: *mut libc::c_char,
) -> *mut libc::c_char {
    libc::memcpy(resolved_path as _, path as _, libc::strlen(path));

    resolved_path
}

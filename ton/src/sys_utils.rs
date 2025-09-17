pub fn sys_tonlib_set_verbosity_level(level: u32) {
    unsafe {
        tonlib_sys::tonlib_client_set_verbosity_level(level);
    }
}

//! The `uptime:mod/host` import, declared by hand.
//!
//! wit-bindgen emits the package segment `mod` of `uptime:mod` as a bare
//! module name, which is a Rust keyword, so the generated import module does
//! not compile. The binding is one function; this is its canonical-ABI shape.

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "uptime:mod/host@0.1.0")]
extern "C" {
    #[link_name = "log"]
    fn wit_import_log(level: i32, ptr: *const u8, len: usize);
}

#[cfg(not(target_arch = "wasm32"))]
unsafe fn wit_import_log(_level: i32, _ptr: *const u8, _len: usize) {}

/// `host.log`: level 0 (trace) to 4 (error). The host caps count and length.
pub fn log(level: u8, msg: &str) {
    // SAFETY: the pointer and length describe `msg`, which outlives the call,
    // and the host copies the bytes before returning.
    unsafe { wit_import_log(i32::from(level), msg.as_ptr(), msg.len()) }
}

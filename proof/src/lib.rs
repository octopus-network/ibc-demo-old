#![no_std]
#![cfg_attr(
    not(feature = "std"),
    feature(core_intrinsics, lang_items, core_panic_info, alloc_error_handler)
)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

#[cfg(not(feature = "std"))]
#[panic_handler]
#[no_mangle]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::intrinsics::abort() }
}

#[cfg(not(feature = "std"))]
#[alloc_error_handler]
#[no_mangle]
pub fn oom(_: core::alloc::Layout) -> ! {
    unsafe {
        core::intrinsics::abort();
    }
}

#[cfg(not(feature = "std"))]
#[no_mangle]
// pub extern "C" fn check_read_proof(params: *const u8, len: usize) -> usize {
pub extern "C" fn check_read_proof() -> i32 {
    // unsafe { ll::ext_check_read_proof() + 1 }
    unsafe { ll::ext_add(1, 2) }
    // 1
}

mod ll {
    extern "C" {
        pub fn ext_add(a: i32, b: i32) -> i32;
        pub fn ext_check_read_proof() -> i32;
    }
}

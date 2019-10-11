#![no_std]
#![feature(core_intrinsics, lang_items, core_panic_info, alloc_error_handler)]

use core::intrinsics;
use core::panic::PanicInfo;

#[cfg(not(feature = "no_panic_handler"))]
#[panic_handler]
#[no_mangle]
pub fn panic(_: &PanicInfo) -> ! {
    unsafe { intrinsics::abort() }
}

#[cfg(not(feature = "no_oom"))]
#[alloc_error_handler]
pub extern "C" fn oom(_: core::alloc::Layout) -> ! {
    unsafe {
        intrinsics::abort();
    }
}

#[no_mangle]
pub extern "C" fn check_read_proof(params: *const u8, len: usize) -> bool {
    unsafe { ll::ext_check_read_proof(params, len) }
}

#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    unsafe { ll::ext_add(a, b) }
}

mod ll {
    extern "C" {
        pub fn ext_add(a: i32, b: i32) -> i32;
        pub fn ext_check_read_proof(params: *const u8, len: usize) -> bool;
    }
}

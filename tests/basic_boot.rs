#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(marginalia::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use marginalia::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    marginalia::test_panic_handler(info)
}

#[test_case]
fn annotation_survives_a_bare_boot() {
    println!("an annotation set with nothing else prepared");
}

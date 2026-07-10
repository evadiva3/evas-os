#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(marginalia::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use marginalia::println;

entry_point!(main);

fn main(_boot_info: &'static BootInfo) -> ! {
    test_main();

    marginalia::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    marginalia::test_panic_handler(info)
}

#[test_case]
fn annotation_survives_a_bare_boot() {
    println!("an annotation set with nothing else prepared");
}

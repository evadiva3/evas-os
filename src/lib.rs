#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

pub mod allocator;
pub mod gdt;
pub mod graphics;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod task;
pub mod vga_buffer;

use core::panic::PanicInfo;

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("examining {} ... ", core::any::type_name::<T>());
        self();
        serial_println!("holds.");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("collation begins: {} leaves to examine.", tests.len());
    for test in tests {
        test.run();
    }
    serial_println!("collation complete; the text stands.");
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("does not hold.");
    match info.location() {
        Some(location) => serial_println!(
            "¶ the text breaks off at {}, line {}, column {}.",
            location.file(),
            location.line(),
            location.column()
        ),
        None => serial_println!("¶ the text breaks off; the place is not recorded."),
    }
    serial_println!("  the scribe's last words: {}", info.message());
    serial_println!("  the collation is abandoned.");
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[test_case]
fn the_obvious_holds() {
    assert_eq!(1, 1);
}

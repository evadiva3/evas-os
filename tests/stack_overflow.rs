#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use marginalia::{exit_qemu, serial_print, serial_println, QemuExitCode};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

entry_point!(main);

fn main(_boot_info: &'static BootInfo) -> ! {
    serial_print!("examining stack_overflow::the_spare_quire_catches_the_fall ... ");

    marginalia::gdt::init();
    init_test_idt();

    stack_overflow();

    panic!("the text ran past the end of the page and nothing stopped it");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read(); // defeats tail-call optimization
}

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(marginalia::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

pub fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("holds.");
    exit_qemu(QemuExitCode::Success);
    marginalia::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    marginalia::test_panic_handler(info)
}

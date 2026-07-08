#![no_std]
#![no_main]

use core::panic::PanicInfo;
use marginalia::{exit_qemu, serial_print, serial_println, QemuExitCode};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("the blot failed to appear; the page is suspiciously clean.");
    exit_qemu(QemuExitCode::Failed);

    loop {}
}

fn should_fail() {
    serial_print!("examining should_panic::should_fail ... ");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("breaks off, as intended.");
    exit_qemu(QemuExitCode::Success);

    loop {}
}

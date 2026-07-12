#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(marginalia::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use marginalia::graphics;
use x86_64::VirtAddr;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    marginalia::init();
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    unsafe { graphics::enter(phys_mem_offset) };
    test_main();
    marginalia::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    marginalia::test_panic_handler(info)
}

const BLOCK: char = '\u{3a9}';

#[test_case]
fn sepia_ink_lands_where_ruled() {
    graphics::write_annotation(format_args!("{}", BLOCK));
    let y = (graphics::ROWS - 2) * 8 + 1;
    assert_eq!(graphics::pixel_at(1, y), graphics::ANNOTATION_INK);
    assert_eq!(graphics::pixel_at(0, y - 1), graphics::PAPER);
}

#[test_case]
fn a_fault_note_is_set_in_yellow() {
    graphics::write_margin_note(format_args!("{}", BLOCK));
    let y = (graphics::ROWS - 2) * 8 + 1;
    assert_eq!(graphics::pixel_at(1, y), graphics::MARGIN_NOTE_INK);
}

#[test_case]
fn the_page_scrolls_without_losing_the_line() {
    graphics::write_annotation(format_args!("{}", BLOCK));
    graphics::write_annotation(format_args!(""));
    let y = (graphics::ROWS - 3) * 8 + 1;
    assert_eq!(graphics::pixel_at(1, y), graphics::ANNOTATION_INK);
}

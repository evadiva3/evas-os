#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(marginalia::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use marginalia::vga_buffer;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use marginalia::allocator;
    use marginalia::memory::{self, BootInfoFrameAllocator};
    use marginalia::task::{executor::Executor, keyboard, Task};
    use x86_64::VirtAddr;

    marginalia::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("the heap could not be bound in");

    unsafe { marginalia::graphics::enter(phys_mem_offset) };

    boot_sequence();

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

fn boot_sequence() {
    vga_buffer::write_annotation(format_args!("MARGINALIA"));
    vga_buffer::write_annotation(format_args!("an annotated machine, begun again"));
    vga_buffer::write_annotation(format_args!(""));
    vga_buffer::write_annotation(format_args!(
        "leaf i.    the processor holds long mode"
    ));
    vga_buffer::write_annotation(format_args!(
        "leaf ii.   forty columns at 0xa0000"
    ));
    vga_buffer::write_annotation(format_args!(
        "leaf iii.  faults become annotations"
    ));
    vga_buffer::write_annotation(format_args!(
        "leaf iv.   a quire kept against doubling"
    ));
    vga_buffer::write_annotation(format_args!(
        "leaf v.    bells at 32; the keys speak"
    ));
    vga_buffer::write_annotation(format_args!(
        "leaf vi.   memory charted from an offset"
    ));
    vga_buffer::write_annotation(format_args!(
        "leaf vii.  a heap at 0x4444_4444_0000"
    ));
    vga_buffer::write_annotation(format_args!(
        "leaf viii. each task writes in its turn"
    ));
    vga_buffer::write_annotation(format_args!(
        "leaf ix.   the ink is true sepia at last"
    ));
    vga_buffer::write_annotation(format_args!(""));
    vga_buffer::write_annotation(format_args!("the margin is quiet."));
    vga_buffer::write_annotation(format_args!("it will note what follows."));
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga_buffer::write_margin_note(format_args!(""));
    match info.location() {
        Some(location) => vga_buffer::write_margin_note(format_args!(
            "¶ the text breaks off at {}, line {}, column {}.",
            location.file(),
            location.line(),
            location.column()
        )),
        None => vga_buffer::write_margin_note(format_args!(
            "¶ the text breaks off; the place is not recorded."
        )),
    }
    vga_buffer::write_margin_note(format_args!(
        "  the scribe's last words: {}",
        info.message()
    ));
    vga_buffer::write_margin_note(format_args!(
        "  nothing past this line survives. the hand rests here."
    ));

    marginalia::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    marginalia::test_panic_handler(info)
}

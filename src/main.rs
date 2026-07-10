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
    use x86_64::VirtAddr;

    marginalia::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("the heap could not be bound in");

    boot_sequence();

    #[cfg(test)]
    test_main();

    marginalia::hlt_loop();
}

fn boot_sequence() {
    vga_buffer::write_annotation(format_args!("MARGINALIA"));
    vga_buffer::write_annotation(format_args!("an annotated machine, begun again"));
    vga_buffer::write_annotation(format_args!(""));
    vga_buffer::write_annotation(format_args!(
        "leaf i.    the processor arrives in long mode; the loader's work holds"
    ));
    vga_buffer::write_annotation(format_args!(
        "leaf ii.   eighty columns ruled at 0xb8000; this ink is the proof"
    ));
    vga_buffer::write_annotation(format_args!(
        "leaf iii.  a table of interruptions is drawn; faults become annotations"
    ));
    vga_buffer::write_annotation(format_args!(
        "leaf iv.   a spare quire stands ready, should a fault double"
    ));
    vga_buffer::write_annotation(format_args!(
        "leaf v.    the bells are rehung at 32; the keyboard is given leave to speak"
    ));
    vga_buffer::write_annotation(format_args!(
        "leaf vi.   the whole of memory is charted from a fixed offset"
    ));
    vga_buffer::write_annotation(format_args!(
        "leaf vii.  a heap is bound in at 0x4444_4444_0000; the text may grow"
    ));
    vga_buffer::write_annotation(format_args!(""));
    vga_buffer::write_annotation(format_args!(
        "the margin is quiet. it will note what follows."
    ));
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

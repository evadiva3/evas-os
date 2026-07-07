

#![no_std]
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    boot_sequence();

    loop {}
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
        "leaf iii.  no interrupts taken, no memory claimed; the page is otherwise blank"
    ));
    vga_buffer::write_annotation(format_args!(""));
    vga_buffer::write_annotation(format_args!(
        "the margin is quiet. it will note what follows."
    ));
}


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

    loop {}
}

use crate::{gdt, print, vga_buffer};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    vga_buffer::write_margin_note(format_args!(
        "¶ a reader's mark (int3) at {:#x}; the text resumes.",
        stack_frame.instruction_pointer.as_u64()
    ));
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    vga_buffer::write_margin_note(format_args!(""));
    vga_buffer::write_margin_note(format_args!("¶ a page is missing from the codex."));
    vga_buffer::write_margin_note(format_args!(
        "  the hand reached for {:#x}, from {:#x}.",
        Cr2::read().as_u64(),
        stack_frame.instruction_pointer.as_u64()
    ));
    vga_buffer::write_margin_note(format_args!(
        "  the nature of the trespass: {:?}.",
        error_code
    ));
    vga_buffer::write_margin_note(format_args!("  the codex waits here, unfinished."));

    crate::hlt_loop();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    vga_buffer::write_margin_note(format_args!(""));
    vga_buffer::write_margin_note(format_args!(
        "¶ the fault doubles; the margin cannot hold it."
    ));
    vga_buffer::write_margin_note(format_args!(
        "  the text broke off at {:#x}.",
        stack_frame.instruction_pointer.as_u64()
    ));
    vga_buffer::write_margin_note(format_args!("  the codex closes here."));

    loop {
        x86_64::instructions::hlt();
    }
}

// the bell rings often and says nothing new; it gets no ink
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore
            ));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

#[test_case]
fn a_readers_mark_does_not_end_the_text() {
    x86_64::instructions::interrupts::int3();
}

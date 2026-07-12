
use core::fmt;
use core::sync::atomic::{AtomicBool, Ordering};
use font8x8::legacy::{BASIC_LEGACY, LATIN_LEGACY};
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::VirtAddr;

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 200;
pub const COLUMNS: usize = WIDTH / 8;
pub const ROWS: usize = HEIGHT / 8;

pub const PAPER: u8 = 0;
pub const ANNOTATION_INK: u8 = 1;
pub const MARGIN_NOTE_INK: u8 = 2;

// true sepia at last: rgb(112, 68, 20), and the fault yellow, in 6-bit DAC steps
const ANNOTATION_RGB: (u8, u8, u8) = (28, 17, 5);
const MARGIN_NOTE_RGB: (u8, u8, u8) = (56, 45, 8);

const FRAMEBUFFER_PHYS: u64 = 0xa0000;

static ACTIVE: AtomicBool = AtomicBool::new(false);
static WRITER: Mutex<Option<Writer>> = Mutex::new(None);

const MISC: u8 = 0x63;
const SEQ: [u8; 5] = [0x03, 0x01, 0x0f, 0x00, 0x0e];
const CRTC: [u8; 25] = [
    0x5f, 0x4f, 0x50, 0x82, 0x54, 0x80, 0xbf, 0x1f, 0x00, 0x41, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x9c, 0x0e, 0x8f, 0x28, 0x40, 0x96, 0xb9, 0xa3, 0xff,
];
const GC: [u8; 9] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x05, 0x0f, 0xff];
const AC: [u8; 21] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c,
    0x0d, 0x0e, 0x0f, 0x41, 0x00, 0x0f, 0x00, 0x00,
];

/// Caller must guarantee the complete physical memory is mapped at
/// `physical_memory_offset`, and that this is called only once.
pub unsafe fn enter(physical_memory_offset: VirtAddr) {
    set_mode_13h();
    set_palette();

    let framebuffer = (physical_memory_offset + FRAMEBUFFER_PHYS).as_mut_ptr::<u8>();
    for i in 0..WIDTH * HEIGHT {
        framebuffer.add(i).write_volatile(PAPER);
    }

    x86_64::instructions::interrupts::without_interrupts(|| {
        *WRITER.lock() = Some(Writer {
            column: 0,
            ink: ANNOTATION_INK,
            framebuffer,
        });
    });
    ACTIVE.store(true, Ordering::Release);
}

pub fn is_active() -> bool {
    ACTIVE.load(Ordering::Acquire)
}

unsafe fn set_mode_13h() {
    let mut misc: Port<u8> = Port::new(0x3c2);
    misc.write(MISC);

    let mut seq_index: Port<u8> = Port::new(0x3c4);
    let mut seq_data: Port<u8> = Port::new(0x3c5);
    for (i, &value) in SEQ.iter().enumerate() {
        seq_index.write(i as u8);
        seq_data.write(value);
    }

    let mut crtc_index: Port<u8> = Port::new(0x3d4);
    let mut crtc_data: Port<u8> = Port::new(0x3d5);
    // unlock CRTC registers 0-7 before rewriting them
    crtc_index.write(0x03);
    let unlocked = crtc_data.read() | 0x80;
    crtc_data.write(unlocked);
    crtc_index.write(0x11);
    let unlocked = crtc_data.read() & !0x80;
    crtc_data.write(unlocked);
    for (i, &value) in CRTC.iter().enumerate() {
        crtc_index.write(i as u8);
        crtc_data.write(value);
    }

    let mut gc_index: Port<u8> = Port::new(0x3ce);
    let mut gc_data: Port<u8> = Port::new(0x3cf);
    for (i, &value) in GC.iter().enumerate() {
        gc_index.write(i as u8);
        gc_data.write(value);
    }

    let mut instat: Port<u8> = Port::new(0x3da);
    let mut ac: Port<u8> = Port::new(0x3c0);
    for (i, &value) in AC.iter().enumerate() {
        instat.read();
        ac.write(i as u8);
        ac.write(value);
    }
    instat.read();
    ac.write(0x20);
}

unsafe fn set_palette() {
    let mut dac_index: Port<u8> = Port::new(0x3c8);
    let mut dac_data: Port<u8> = Port::new(0x3c9);
    dac_index.write(0);
    for i in 0..=255u8 {
        let (r, g, b) = match i {
            ANNOTATION_INK => ANNOTATION_RGB,
            MARGIN_NOTE_INK => MARGIN_NOTE_RGB,
            _ => (0, 0, 0),
        };
        dac_data.write(r);
        dac_data.write(g);
        dac_data.write(b);
    }
}

struct Writer {
    column: usize,
    ink: u8,
    framebuffer: *mut u8,
}

// the single framebuffer pointer is only ever reached through the WRITER mutex
unsafe impl Send for Writer {}

const FALLBACK_GLYPH: [u8; 8] = [0x00, 0x7e, 0x7e, 0x7e, 0x7e, 0x7e, 0x7e, 0x00];

fn glyph(c: char) -> [u8; 8] {
    match c as u32 {
        code @ 0x00..=0x7f => BASIC_LEGACY[code as usize],
        code @ 0xa0..=0xff => LATIN_LEGACY[code as usize - 0xa0],
        _ => FALLBACK_GLYPH,
    }
}

impl Writer {
    fn write_char(&mut self, c: char) {
        if c == '\n' {
            self.new_line();
            return;
        }
        if self.column >= COLUMNS {
            self.new_line();
        }

        let glyph = glyph(c);
        let x0 = self.column * 8;
        let y0 = (ROWS - 1) * 8;
        for (dy, &row_bits) in glyph.iter().enumerate() {
            for dx in 0..8 {
                let color = if row_bits & (1 << dx) != 0 {
                    self.ink
                } else {
                    PAPER
                };
                self.plot(x0 + dx, y0 + dy, color);
            }
        }
        self.column += 1;
    }

    fn plot(&mut self, x: usize, y: usize, color: u8) {
        unsafe {
            self.framebuffer.add(y * WIDTH + x).write_volatile(color);
        }
    }

    fn new_line(&mut self) {
        unsafe {
            for i in 0..WIDTH * (HEIGHT - 8) {
                let above = self.framebuffer.add(i + WIDTH * 8).read_volatile();
                self.framebuffer.add(i).write_volatile(above);
            }
            for i in WIDTH * (HEIGHT - 8)..WIDTH * HEIGHT {
                self.framebuffer.add(i).write_volatile(PAPER);
            }
        }
        self.column = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}

fn with_ink(ink: u8, args: fmt::Arguments, end_line: bool) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        if let Some(writer) = WRITER.lock().as_mut() {
            writer.ink = ink;
            writer.write_fmt(args).unwrap();
            if end_line {
                writer.write_char('\n');
            }
            writer.ink = ANNOTATION_INK;
        }
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    with_ink(ANNOTATION_INK, args, false);
}

pub fn write_annotation(args: fmt::Arguments) {
    with_ink(ANNOTATION_INK, args, true);
}

pub fn write_margin_note(args: fmt::Arguments) {
    with_ink(MARGIN_NOTE_INK, args, true);
}

pub fn pixel_at(x: usize, y: usize) -> u8 {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        let writer = WRITER.lock();
        let writer = writer.as_ref().expect("the page is not yet painted");
        unsafe { writer.framebuffer.add(y * WIDTH + x).read_volatile() }
    })
}

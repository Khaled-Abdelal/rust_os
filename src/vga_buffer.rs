// VGA chars are 16 bit. 8 for the char and 8 for the color + blink option

// define the differ ent colors as an enum
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // add copy semantics (like premative types) and make it printable and comparable
#[repr(u8)] // force the enum's discriminant (the value used internally to represent each variant)
            // to be a u8. Bye default they are stored as isize (32 bits on 32 bit systems and 64 bits on 64 bit systems)
            //  4bit is enough but rust doesn't have u4
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

// define the color including foreground and background
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // this ensures the struct  layout in memory is the same as the type u8 this
                     // can be useful for FFI (Foreign Function Interface) with C code.
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        // the color code is a u8 where the first 4 bits are the background color and the last 4 bits are the foreground color
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // ensure the ordering of the fields is the same as in C. The default ordering is undefined
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

// a VGA buffer is a 2D array of 25 rows and 80 columns
const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

use volatile::Volatile; // if we don't read the written values the compiler might optimize it away so we use volatile to prevent that
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// To actually write to the screen we define a writer struct
// the writer always writes to the last row
// The static lifetime is required, we specify static because the buffer (VGA buffer) lives for the entire duration of the program
pub struct Writer {
    color_code: ColorCode,
    column_position: usize,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // rust strings are utf8 so we need to write only printable ASCII bytes or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe), // write a ■ character for unprintable bytes
            }
        }
    }

    fn new_line(&mut self) {
        // move all the lines up one row
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character)
            }
        }
        // empty current row
        self.clear_row(BUFFER_HEIGHT - 1);
        // move the cursor to the beginning of the row
        self.column_position = 0
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

// Implement the rust fmt write so we can easily use the write! macro and print different types
use core::fmt;
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        return Ok(());
    }
}

// example of how to use the writer
// pub fn print_something() {
//     use core::fmt::Write;
//     let mut writer = Writer {
//         column_position: 0,
//         color_code: ColorCode::new(Color::Yellow, Color::Black),
//         // raw pointer to the VGA buffer. The unsafe block is needed because we are dereferencing a raw pointer
//         buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
//     };
//
//     writer.write_byte(b'H');
//     writer.write_string("ello ");
//     writer.write_string("Wörld!");
//     writer.write_byte(b'\n');
//     write!(writer, "Formatted string {} with number", 42).unwrap();
// }

/*
static variables are computed at compiles time, but rust can't compute raw pointers at compile
time so we use the lazy_static crate which force the writer to be created at runtime.

The writer wouldn't be useful if it's immutable since the internal functions accept &mut self
but making the static global variable mutable is unsafe and discouraged (will cause race conditions)
To go around this we use the INTERIOR_MUTABILITY_PATTERN in rust which gives us the ability to mutate the resource with immutable references
We use SPINLOCKS to ensure that only one thread can access the resource at a time
*/
use lazy_static::lazy_static;
use spin::Mutex;
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// define our own !prinln macro, they are copied from rust's defintion with only a change to use
// the VGA write
// #[macro_export] makes the macro available for the whole crate
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

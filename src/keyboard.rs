const KEYBOARD_DATA_PORT: u16 = 0x60;
const KEYBOARD_STATUS_PORT: u16 = 0x64;

pub struct Keyboard;

impl Keyboard {
    pub fn wait_and_read_scancode() -> u8 {
        unsafe {
            loop {
                let mut status: u8;
                core::arch::asm!(
                    "in al, dx",
                    in("dx") KEYBOARD_STATUS_PORT,
                    out("al") status,
                    options(nomem, nostack)
                );
                if status & 1 != 0 {
                    break;
                }
            }
            
            let mut value: u8;
            core::arch::asm!(
                "in al, dx",
                in("dx") KEYBOARD_DATA_PORT,
                out("al") value,
                options(nomem, nostack)
            );
            value
        }
    }

    pub fn scancode_to_char(scancode: u8, shift: bool) -> Option<char> {
        match scancode {
            0x1E => Some(if shift { 'A' } else { 'a' }),
            0x30 => Some(if shift { 'B' } else { 'b' }),
            0x2E => Some(if shift { 'C' } else { 'c' }),
            0x20 => Some(if shift { 'D' } else { 'd' }),
            0x12 => Some(if shift { 'E' } else { 'e' }),
            0x21 => Some(if shift { 'F' } else { 'f' }),
            0x22 => Some(if shift { 'G' } else { 'g' }),
            0x23 => Some(if shift { 'H' } else { 'h' }),
            0x17 => Some(if shift { 'I' } else { 'i' }),
            0x24 => Some(if shift { 'J' } else { 'j' }),
            0x25 => Some(if shift { 'K' } else { 'k' }),
            0x26 => Some(if shift { 'L' } else { 'l' }),
            0x32 => Some(if shift { 'M' } else { 'm' }),
            0x31 => Some(if shift { 'N' } else { 'n' }),
            0x18 => Some(if shift { 'O' } else { 'o' }),
            0x19 => Some(if shift { 'P' } else { 'p' }),
            0x10 => Some(if shift { 'Q' } else { 'q' }),
            0x13 => Some(if shift { 'R' } else { 'r' }),
            0x1F => Some(if shift { 'S' } else { 's' }),
            0x14 => Some(if shift { 'T' } else { 't' }),
            0x16 => Some(if shift { 'U' } else { 'u' }),
            0x2F => Some(if shift { 'V' } else { 'v' }),
            0x11 => Some(if shift { 'W' } else { 'w' }),
            0x2D => Some(if shift { 'X' } else { 'x' }),
            0x15 => Some(if shift { 'Y' } else { 'y' }),
            0x2C => Some(if shift { 'Z' } else { 'z' }),
            0x02 => Some(if shift { '!' } else { '1' }),
            0x03 => Some(if shift { '@' } else { '2' }),
            0x04 => Some(if shift { '#' } else { '3' }),
            0x05 => Some(if shift { '$' } else { '4' }),
            0x06 => Some(if shift { '%' } else { '5' }),
            0x07 => Some(if shift { '^' } else { '6' }),
            0x08 => Some(if shift { '&' } else { '7' }),
            0x09 => Some(if shift { '*' } else { '8' }),
            0x0A => Some(if shift { '(' } else { '9' }),
            0x0B => Some(if shift { ')' } else { '0' }),
            0x39 => Some(' '),
            0x0C => Some(if shift { '_' } else { '-' }),
            0x0D => Some(if shift { '+' } else { '=' }),
            0x1A => Some(if shift { '{' } else { '[' }),
            0x1B => Some(if shift { '}' } else { ']' }),
            0x27 => Some(if shift { ':' } else { ';' }),
            0x28 => Some(if shift { '"' } else { '\'' }),
            0x33 => Some(if shift { '<' } else { ',' }),
            0x34 => Some(if shift { '>' } else { '.' }),
            0x35 => Some(if shift { '?' } else { '/' }),
            0x29 => Some(if shift { '~' } else { '`' }),
            0x2B => Some(if shift { '|' } else { '\\' }),
            _ => None,
        }
    }
}

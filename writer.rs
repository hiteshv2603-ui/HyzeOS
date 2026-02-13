use crate::vga_colors::{Color, color_code};
use core::arch::asm;

const VGA_BUFFER: usize = 0xb8000;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

const VGA_CTRL_PORT: u16 = 0x3D4;
const VGA_DATA_PORT: u16 = 0x3D5;

pub struct Writer {
    col: usize,
    row: usize,
    color: u8,
}

impl Writer {
    pub const fn new(color: u8) -> Self {
        Self { col: 0, row: 0, color }
    }

    fn update_cursor(&self) {
        let pos = self.row * VGA_WIDTH + self.col;
        unsafe {
            outb(VGA_CTRL_PORT, 0x0F);
            outb(VGA_DATA_PORT, (pos & 0xFF) as u8);

            outb(VGA_CTRL_PORT, 0x0E);
            outb(VGA_DATA_PORT, ((pos >> 8) & 0xFF) as u8);
        }
    }

    pub fn enable_cursor(&self) {
        unsafe {
            outb(VGA_CTRL_PORT, 0x0A);
            outb(VGA_DATA_PORT, 14);

            outb(VGA_CTRL_PORT, 0x0B);
            outb(VGA_DATA_PORT, 15);
        }
        self.update_cursor();
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            byte => {
                if self.col >= VGA_WIDTH {
                    self.newline();
                }

                let offset = (self.row * VGA_WIDTH + self.col) * 2;
                unsafe {
                    let vga = VGA_BUFFER as *mut u8;
                    *vga.add(offset) = byte;
                    *vga.add(offset + 1) = self.color;
                }
                self.col += 1;
            }
        }
        self.update_cursor();
    }

    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_bytes(&mut self, s: &[u8]) {
        for &byte in s {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn newline(&mut self) {
        self.col = 0;
        if self.row < VGA_HEIGHT - 1 {
            self.row += 1;
        } else {
            self.scroll();
        }
    }

    fn scroll(&mut self) {
        unsafe {
            let vga = VGA_BUFFER as *mut u8;
            for row in 1..VGA_HEIGHT {
                for col in 0..VGA_WIDTH {
                    let src = (row * VGA_WIDTH + col) * 2;
                    let dst = ((row - 1) * VGA_WIDTH + col) * 2;
                    *vga.add(dst) = *vga.add(src);
                    *vga.add(dst + 1) = *vga.add(src + 1);
                }
            }
            for col in 0..VGA_WIDTH {
                let offset = ((VGA_HEIGHT - 1) * VGA_WIDTH + col) * 2;
                *vga.add(offset) = b' ';
                *vga.add(offset + 1) = self.color;
            }
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            let vga = VGA_BUFFER as *mut u8;
            for i in 0..(VGA_WIDTH * VGA_HEIGHT) {
                *vga.add(i * 2) = b' ';
                *vga.add(i * 2 + 1) = self.color;
            }
        }
        self.col = 0;
        self.row = 0;
    }

    pub fn set_color(&mut self, fg: Color, bg: Color) {
        self.color = color_code(fg, bg);
    }

    pub fn get_col(&self) -> usize {
        self.col
    }

    pub fn get_row(&self) -> usize {
        self.row
    }

    pub fn set_position(&mut self, col: usize, row: usize) {
        self.col = col;
        self.row = row;
        self.update_cursor();
    }
}

unsafe fn outb(port: u16, value: u8) {
    asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nostack)
    );
}

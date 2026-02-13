#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xA,
    LightCyan = 0xB,
    LightRed = 0xC,
    Pink = 0xD,
    Yellow = 0xE,
    White = 0xF,
}

#[allow(dead_code)]
pub const fn color_code(foreground: Color, background: Color) -> u8 {
    (background as u8) << 4 | (foreground as u8)
}

#[allow(dead_code)]
pub mod presets {
    use super::{color_code, Color};

    pub const DEFAULT: u8 = color_code(Color::LightGray, Color::Black);
    pub const ERROR: u8 = color_code(Color::LightRed, Color::Black);
    pub const WARNING: u8 = color_code(Color::Yellow, Color::Black);
    pub const SUCCESS: u8 = color_code(Color::LightGreen, Color::Black);
    pub const INFO: u8 = color_code(Color::LightCyan, Color::Black);
    pub const HIGHLIGHT: u8 = color_code(Color::Black, Color::White);
    pub const ADER: u8 = color_code(Color::White, Color::Blue);
}

use crate::writer::Writer;
use crate::keyboard::Keyboard;
use crate::vga_colors::Color;
use crate::idt;
use crate::filesystem::get_filesystem;

const EDITOR_WIDTH: usize = 80;
const EDITOR_HEIGHT: usize = 23;
const MAX_LINES: usize = 100;
const MAX_LINE_LEN: usize = 80;

pub struct Editor {
    lines: [[u8; MAX_LINE_LEN]; MAX_LINES],
    line_lengths: [usize; MAX_LINES],
    num_lines: usize,
    cursor_x: usize,
    cursor_y: usize,
    scroll_offset: usize,
    modified: bool,
    filename: [u8; 32],
    filename_len: usize,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            lines: [[0; MAX_LINE_LEN]; MAX_LINES],
            line_lengths: [0; MAX_LINES],
            num_lines: 1,
            cursor_x: 0,
            cursor_y: 0,
            scroll_offset: 0,
            modified: false,
            filename: [0; 32],
            filename_len: 0,
        }
    }

    pub fn open(&mut self, filename: &[u8]) -> Result<(), &'static str> {
        if filename.len() > 32 {
            return Err("Filename too long");
        }

        self.filename[..filename.len()].copy_from_slice(filename);
        self.filename_len = filename.len();

        let fs = get_filesystem();
        if let Some(content) = fs.read_file(filename) {
            self.load_content(content);
        } else {
            self.num_lines = 1;
            self.line_lengths[0] = 0;
        }

        Ok(())
    }

    fn load_content(&mut self, content: &[u8]) {
        self.num_lines = 0;
        let mut line_idx = 0;
        let mut col = 0;

        for &byte in content {
            if byte == b'\n' {
                self.line_lengths[line_idx] = col;
                line_idx += 1;
                col = 0;
                if line_idx >= MAX_LINES {
                    break;
                }
            } else if col < MAX_LINE_LEN {
                self.lines[line_idx][col] = byte;
                col += 1;
            }
        }

        if line_idx < MAX_LINES {
            self.line_lengths[line_idx] = col;
            self.num_lines = line_idx + 1;
        } else {
            self.num_lines = MAX_LINES;
        }

        self.modified = false;
    }

    pub fn run(&mut self, writer: &mut Writer) {
        let mut shift_pressed = false;
        let mut ctrl_pressed = false;
        
        writer.clear();
        
        self.draw(writer);

        loop {
            let scancode = match idt::get_scancode() {
                Some(sc) => sc,
                None => {
                    idt::wait_for_interrupt();
                    continue;
                }
            };

            if scancode & 0x80 != 0 {
                if scancode == 0xAA || scancode == 0xB6 {
                    shift_pressed = false;
                }
                if scancode == 0x9D {
                    ctrl_pressed = false;
                }
                continue;
            }

            if scancode == 0x2A || scancode == 0x36 {
                shift_pressed = true;
                continue;
            }

            if scancode == 0x1D {
                ctrl_pressed = true;
                continue;
            }

            if ctrl_pressed && scancode == 0x1F {
                if let Err(e) = self.save() {
                    self.show_message(writer, e, Color::Red);
                } else {
                    self.show_message(writer, "Saved!", Color::Green);
                }
                self.draw(writer);
                continue;
            }

            if ctrl_pressed && scancode == 0x10 {
                if self.modified {
                    self.show_message(writer, "Unsaved changes! Press Ctrl+Q again to quit", Color::Yellow);
                    self.draw(writer);
                    let scancode2 = self.wait_for_key();
                    if scancode2 == 0x10 && ctrl_pressed {
                        break;
                    }
                    continue;
                } else {
                    break;
                }
            }

            if scancode == 0x01 {
                if self.modified {
                    self.show_message(writer, "Unsaved changes! Press ESC again to quit", Color::Yellow);
                    self.draw(writer);
                    let scancode2 = self.wait_for_key();
                    if scancode2 == 0x01 {
                        break;
                    }
                    continue;
                } else {
                    break;
                }
            }

            let needs_redraw = match scancode {
                0x48 => { self.move_up(); true }
                0x50 => { self.move_down(); true }
                0x4B => { self.move_left(); true }
                0x4D => { self.move_right(); true }
                0x47 => { self.home(); true }
                0x4F => { self.end(); true }
                0x49 => { self.page_up(); true }
                0x51 => { self.page_down(); true }
                _ => false
            };

            if needs_redraw {
                self.draw(writer);
                continue;
            }

            if scancode == 0x1C {
                self.insert_newline();
                self.draw(writer);
                continue;
            }

            if scancode == 0x0E {
                self.backspace();
                self.draw(writer);
                continue;
            }

            if let Some(c) = Keyboard::scancode_to_char(scancode, shift_pressed) {
                if !ctrl_pressed {
                    self.insert_char(c as u8);
                    self.draw(writer);
                }
            }
        }
    }

    fn draw(&self, writer: &mut Writer) {
        writer.set_position(0, 0);

        for screen_row in 0..EDITOR_HEIGHT {
            let file_row = screen_row + self.scroll_offset;
            
            if file_row < self.num_lines {
                let line_len = self.line_lengths[file_row];
                writer.set_color(Color::White, Color::Black);
                writer.write_bytes(&self.lines[file_row][..line_len]);
                for _ in line_len..EDITOR_WIDTH {
                    writer.write_byte(b' ');
                }
            } else {
                writer.set_color(Color::White, Color::Black);
                for _ in 0..EDITOR_WIDTH {
                    writer.write_byte(b' ');
                }
            }
            
            writer.write_byte(b'\n');
        }

        writer.set_color(Color::Black, Color::LightGray);
        let mut status = [b' '; EDITOR_WIDTH];
        
        let mut pos = 1;
        status[pos..pos + self.filename_len].copy_from_slice(&self.filename[..self.filename_len]);
        pos += self.filename_len;
        
        if self.modified {
            status[pos] = b'*';
            pos += 1;
        }

        let pos_str = b" Line:";
        let line_num = self.cursor_y + 1;
        let col_num = self.cursor_x + 1;
        
        let mut temp = [0u8; 10];
        let mut temp_len = 0;
        let mut n = line_num;
        loop {
            temp[temp_len] = (n % 10) as u8 + b'0';
            temp_len += 1;
            n /= 10;
            if n == 0 { break; }
        }
        
        let start_pos = EDITOR_WIDTH - temp_len - 15;
        status[start_pos..start_pos + pos_str.len()].copy_from_slice(pos_str);
        let mut p = start_pos + pos_str.len();
        for i in (0..temp_len).rev() {
            status[p] = temp[i];
            p += 1;
        }
        
        status[p] = b' ';
        p += 1;
        status[p] = b'C';
        p += 1;
        status[p] = b'o';
        p += 1;
        status[p] = b'l';
        p += 1;
        status[p] = b':';
        p += 1;
        
        temp_len = 0;
        n = col_num;
        loop {
            temp[temp_len] = (n % 10) as u8 + b'0';
            temp_len += 1;
            n /= 10;
            if n == 0 { break; }
        }
        for i in (0..temp_len).rev() {
            status[p] = temp[i];
            p += 1;
        }

        writer.write_bytes(&status);
        writer.write_byte(b'\n');

        writer.set_color(Color::White, Color::Black);
        writer.write_str("Ctrl+S: Save | Ctrl+Q or ESC: Quit | Arrows: Navigate");

        let screen_y = self.cursor_y.saturating_sub(self.scroll_offset);
        writer.set_position(self.cursor_x, screen_y);
    }

    fn show_message(&self, writer: &mut Writer, msg: &str, color: Color) {
        writer.set_position(0, 24);
        writer.set_color(color, Color::Black);
        writer.write_str(msg);
        
        for _ in 0..10 {
            idt::wait_for_interrupt();
        }
    }

    fn wait_for_key(&self) -> u8 {
        loop {
            if let Some(sc) = idt::get_scancode() {
                if sc & 0x80 == 0 {
                    return sc;
                }
            }
            idt::wait_for_interrupt();
        }
    }

    fn insert_char(&mut self, c: u8) {
        if self.cursor_y >= MAX_LINES {
            return;
        }

        let line_len = self.line_lengths[self.cursor_y];
        if line_len >= MAX_LINE_LEN {
            return;
        }

        for i in (self.cursor_x..line_len).rev() {
            self.lines[self.cursor_y][i + 1] = self.lines[self.cursor_y][i];
        }

        self.lines[self.cursor_y][self.cursor_x] = c;
        self.line_lengths[self.cursor_y] += 1;
        self.cursor_x += 1;
        self.modified = true;
    }

    fn insert_newline(&mut self) {
        if self.num_lines >= MAX_LINES {
            return;
        }

        for i in (self.cursor_y + 1..self.num_lines).rev() {
            self.lines[i + 1] = self.lines[i];
            self.line_lengths[i + 1] = self.line_lengths[i];
        }

        let current_len = self.line_lengths[self.cursor_y];
        let remaining_len = current_len - self.cursor_x;

        if remaining_len > 0 {
            for i in 0..remaining_len {
                self.lines[self.cursor_y + 1][i] = self.lines[self.cursor_y][self.cursor_x + i];
            }
        }

        self.line_lengths[self.cursor_y + 1] = remaining_len;
        self.line_lengths[self.cursor_y] = self.cursor_x;

        self.num_lines += 1;
        self.cursor_y += 1;
        self.cursor_x = 0;

        if self.cursor_y - self.scroll_offset >= EDITOR_HEIGHT {
            self.scroll_offset += 1;
        }

        self.modified = true;
    }

    fn backspace(&mut self) {
        if self.cursor_x > 0 {
            let line_len = self.line_lengths[self.cursor_y];
            for i in self.cursor_x..line_len {
                self.lines[self.cursor_y][i - 1] = self.lines[self.cursor_y][i];
            }
            self.line_lengths[self.cursor_y] -= 1;
            self.cursor_x -= 1;
            self.modified = true;
        } else if self.cursor_y > 0 {
            let prev_len = self.line_lengths[self.cursor_y - 1];
            let curr_len = self.line_lengths[self.cursor_y];

            if prev_len + curr_len <= MAX_LINE_LEN {
                for i in 0..curr_len {
                    self.lines[self.cursor_y - 1][prev_len + i] = self.lines[self.cursor_y][i];
                }
                self.line_lengths[self.cursor_y - 1] = prev_len + curr_len;

                for i in self.cursor_y..self.num_lines - 1 {
                    self.lines[i] = self.lines[i + 1];
                    self.line_lengths[i] = self.line_lengths[i + 1];
                }

                self.num_lines -= 1;
                self.cursor_y -= 1;
                self.cursor_x = prev_len;

                if self.cursor_y < self.scroll_offset {
                    self.scroll_offset = self.cursor_y;
                }

                self.modified = true;
            }
        }
    }

    fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            let line_len = self.line_lengths[self.cursor_y];
            if self.cursor_x > line_len {
                self.cursor_x = line_len;
            }
            if self.cursor_y < self.scroll_offset {
                self.scroll_offset = self.cursor_y;
            }
        }
    }

    fn move_down(&mut self) {
        if self.cursor_y + 1 < self.num_lines {
            self.cursor_y += 1;
            let line_len = self.line_lengths[self.cursor_y];
            if self.cursor_x > line_len {
                self.cursor_x = line_len;
            }
            if self.cursor_y - self.scroll_offset >= EDITOR_HEIGHT {
                self.scroll_offset += 1;
            }
        }
    }

    fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.line_lengths[self.cursor_y];
            if self.cursor_y < self.scroll_offset {
                self.scroll_offset = self.cursor_y;
            }
        }
    }

    fn move_right(&mut self) {
        let line_len = self.line_lengths[self.cursor_y];
        if self.cursor_x < line_len {
            self.cursor_x += 1;
        } else if self.cursor_y + 1 < self.num_lines {
            self.cursor_y += 1;
            self.cursor_x = 0;
            if self.cursor_y - self.scroll_offset >= EDITOR_HEIGHT {
                self.scroll_offset += 1;
            }
        }
    }

    fn home(&mut self) {
        self.cursor_x = 0;
    }

    fn end(&mut self) {
        self.cursor_x = self.line_lengths[self.cursor_y];
    }

    fn page_up(&mut self) {
        for _ in 0..EDITOR_HEIGHT {
            self.move_up();
        }
    }

    fn page_down(&mut self) {
        for _ in 0..EDITOR_HEIGHT {
            self.move_down();
        }
    }

    fn save(&mut self) -> Result<(), &'static str> {
        let mut content = [0u8; 4096];
        let mut pos = 0;

        for i in 0..self.num_lines {
            let line_len = self.line_lengths[i];
            if pos + line_len + 1 > content.len() {
                return Err("Content too large");
            }

            content[pos..pos + line_len].copy_from_slice(&self.lines[i][..line_len]);
            pos += line_len;

            if i + 1 < self.num_lines {
                content[pos] = b'\n';
                pos += 1;
            }
        }

        let fs = get_filesystem();
        fs.write_file(&self.filename[..self.filename_len], &content[..pos])?;
        self.modified = false;

        Ok(())
    }
}

use crate::writer::Writer;
use crate::vga_colors::Color;
use crate::idt;

const VGA_BUFFER: usize = 0xb8000;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

pub struct VideoPlayer {
    current_frame: usize,
    playing: bool,
    frame_data: &'static [u8],
    frame_count: usize,
    frame_width: usize,
    frame_height: usize,
    target_fps: u32,
}

impl VideoPlayer {
    pub fn new(
        frame_data: &'static [u8],
        frame_count: usize,
        frame_width: usize,
        frame_height: usize,
        target_fps: u32,
    ) -> Self {
        Self {
            current_frame: 0,
            playing: true,
            frame_data,
            frame_count,
            frame_width,
            frame_height,
            target_fps,
        }
    }

    fn get_frame_data(&self, index: usize) -> Option<&[u8]> {
        if index >= self.frame_count {
            return None;
        }
        
        let frame_size = self.frame_width * self.frame_height;
        let start = index * frame_size;
        
        if start + frame_size > self.frame_data.len() {
            return None;
        }
        
        Some(&self.frame_data[start..start + frame_size])
    }

    fn draw_frame(&self, frame_data: &[u8]) {
        unsafe {
            let vga = VGA_BUFFER as *mut u8;
            
            let offset_x = if self.frame_width < VGA_WIDTH { (VGA_WIDTH - self.frame_width) / 2 } else { 0 };
            let offset_y = if self.frame_height < VGA_HEIGHT { (VGA_HEIGHT - self.frame_height) / 2 } else { 0 };
            
            for y in 0..self.frame_height.min(VGA_HEIGHT) {
                for x in 0..self.frame_width.min(VGA_WIDTH) {
                    let src_idx = y * self.frame_width + x;
                    
                    if src_idx >= frame_data.len() {
                        continue;
                    }
                    
                    let ch = frame_data[src_idx];
                    
                    let color = match ch {
                        b' ' | b'.' => 0x08,  // Dark gray
                        b':' | b'-' => 0x07,  // Light gray
                        b'=' | b'+' => 0x0F,  // White
                        b'*' | b'#' => 0x0F,  // White
                        b'%' | b'@' => 0x0F,  // White
                        _ => 0x07,
                    };
                    
                    let vga_x = offset_x + x;
                    let vga_y = offset_y + y;
                    
                    let vga_offset = (vga_y * VGA_WIDTH + vga_x) * 2;
                    
                    *vga.add(vga_offset) = ch;
                    *vga.add(vga_offset + 1) = color;
                }
            }
        }
    }

    fn clear_screen(&self) {
        unsafe {
            let vga = VGA_BUFFER as *mut u8;
            for i in 0..(VGA_WIDTH * VGA_HEIGHT) {
                *vga.add(i * 2) = b' ';
                *vga.add(i * 2 + 1) = 0x00;
            }
        }
    }

    fn draw_progress(&self) {
        unsafe {
            let vga = VGA_BUFFER as *mut u8;
            
            let progress = (self.current_frame * VGA_WIDTH) / self.frame_count;
            let y = VGA_HEIGHT - 1;
            
            for x in 0..VGA_WIDTH {
                let offset = (y * VGA_WIDTH + x) * 2;
                if x < progress {
                    *vga.add(offset) = b'=';
                    *vga.add(offset + 1) = 0x0A;
                } else {
                    *vga.add(offset) = b'-';
                    *vga.add(offset + 1) = 0x08; 
                }
            }
        }
    }

    pub fn run(&mut self) {
        idt::flush_buffer();
        self.clear_screen();
        
        let ticks_per_frame = 100 / self.target_fps;
        let mut last_tick = idt::get_ticks();
        
        loop {
            while let Some(scancode) = idt::get_scancode() {
                if scancode & 0x80 != 0 {
                    continue; 
                }
                
                match scancode {
                    0x10 => return,
                    0x39 => self.playing = !self.playing, 
                    0x4B => {
                        if self.current_frame > 10 {
                            self.current_frame -= 10;
                        } else {
                            self.current_frame = 0;
                        }
                    }
                    0x4D => {
                        self.current_frame = (self.current_frame + 10).min(self.frame_count - 1);
                    }
                    0x47 => self.current_frame = 0,
                    _ => {}
                }
            }
            
            let current_tick = idt::get_ticks();
            if self.playing && current_tick.wrapping_sub(last_tick) >= ticks_per_frame {
                last_tick = current_tick;
                
                if let Some(frame_data) = self.get_frame_data(self.current_frame) {
                    self.draw_frame(frame_data);
                    self.draw_progress();
                }
                
                self.current_frame += 1;
                if self.current_frame >= self.frame_count {
                    self.current_frame = 0;
                }
            }
            
            idt::wait_for_interrupt();
        }
    }
}

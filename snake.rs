use crate::writer::Writer;
use crate::vga_colors::Color;
use crate::idt;

const VGA_BUFFER: usize = 0xb8000;
const GAME_WIDTH: usize = 40;
const GAME_HEIGHT: usize = 20;
const GAME_OFFSET_X: usize = 20;
const GAME_OFFSET_Y: usize = 2;
const MAX_SNAKE_LEN: usize = 100;

#[derive(Copy, Clone, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, PartialEq)]
struct Point {
    x: usize,
    y: usize,
}

pub struct SnakeGame {
    snake: [Point; MAX_SNAKE_LEN],
    snake_len: usize,
    direction: Direction,
    food: Point,
    score: u32,
    game_over: bool,
    seed: u32,
    started: bool,
}

impl SnakeGame {
    pub fn new() -> Self {
        let mut game = Self {
            snake: [Point { x: 0, y: 0 }; MAX_SNAKE_LEN],
            snake_len: 3,
            direction: Direction::Right,
            food: Point { x: 15, y: 10 },
            score: 0,
            game_over: false,
            seed: 12345,
            started: false,
        };

        game.snake[0] = Point { x: 20, y: 10 };
        game.snake[1] = Point { x: 19, y: 10 };
        game.snake[2] = Point { x: 18, y: 10 };
        game
    }

    fn random(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        self.seed
    }

    fn spawn_food(&mut self) {
        loop {
            let x = (self.random() as usize) % GAME_WIDTH;
            let y = (self.random() as usize) % GAME_HEIGHT;
            
            let mut on_snake = false;
            for i in 0..self.snake_len {
                if self.snake[i].x == x && self.snake[i].y == y {
                    on_snake = true;
                    break;
                }
            }
            
            if !on_snake {
                self.food = Point { x, y };
                break;
            }
        }
    }

    fn draw_box(&self) {
        unsafe {
            let vga = VGA_BUFFER as *mut u8;
            let border_color = 0x0F;
            
            for x in 0..=GAME_WIDTH + 1 {
                let top_offset = ((GAME_OFFSET_Y - 1) * 80 + GAME_OFFSET_X - 1 + x) * 2;
                let bottom_offset = ((GAME_OFFSET_Y + GAME_HEIGHT) * 80 + GAME_OFFSET_X - 1 + x) * 2;
                *vga.add(top_offset) = b'#';
                *vga.add(top_offset + 1) = border_color;
                *vga.add(bottom_offset) = b'#';
                *vga.add(bottom_offset + 1) = border_color;
            }
            
            for y in 0..GAME_HEIGHT {
                let left_offset = ((GAME_OFFSET_Y + y) * 80 + GAME_OFFSET_X - 1) * 2;
                let right_offset = ((GAME_OFFSET_Y + y) * 80 + GAME_OFFSET_X + GAME_WIDTH) * 2;
                *vga.add(left_offset) = b'#';
                *vga.add(left_offset + 1) = border_color;
                *vga.add(right_offset) = b'#';
                *vga.add(right_offset + 1) = border_color;
            }
        }
    }

    fn draw_cell(&self, x: usize, y: usize, ch: u8, color: u8) {
        if x < GAME_WIDTH && y < GAME_HEIGHT {
            unsafe {
                let vga = VGA_BUFFER as *mut u8;
                let offset = ((GAME_OFFSET_Y + y) * 80 + GAME_OFFSET_X + x) * 2;
                *vga.add(offset) = ch;
                *vga.add(offset + 1) = color;
            }
        }
    }

    fn clear_game_area(&self) {
        for y in 0..GAME_HEIGHT {
            for x in 0..GAME_WIDTH {
                self.draw_cell(x, y, b' ', 0x00);
            }
        }
    }

    fn draw(&self) {
        for i in 0..self.snake_len {
            let ch = if i == 0 { b'@' } else { b'o' };
            let color = 0x0A;
            self.draw_cell(self.snake[i].x, self.snake[i].y, ch, color);
        }
        
        self.draw_cell(self.food.x, self.food.y, b'*', 0x0C);
        
        unsafe {
            let vga = VGA_BUFFER as *mut u8;
            let score_str = b"Score: ";
            for (i, &byte) in score_str.iter().enumerate() {
                let offset = i * 2;
                *vga.add(offset) = byte;
                *vga.add(offset + 1) = 0x0E;
            }

            let mut score = self.score;
            let mut digits = [0u8; 10];
            let mut digit_count = 0;
            if score == 0 {
                digits[0] = b'0';
                digit_count = 1;
            } else {
                while score > 0 {
                    digits[digit_count] = b'0' + (score % 10) as u8;
                    score /= 10;
                    digit_count += 1;
                }
            }
            for i in 0..digit_count {
                let offset = (7 + i) * 2;
                *vga.add(offset) = digits[digit_count - 1 - i];
                *vga.add(offset + 1) = 0x0E;
            }
        }
    }

    fn update(&mut self) -> bool {
        if self.game_over {
            return false;
        }

        let head = self.snake[0];
        let new_head = match self.direction {
            Direction::Up => {
                if head.y == 0 {
                    self.game_over = true;
                    return false;
                }
                Point { x: head.x, y: head.y - 1 }
            }
            Direction::Down => {
                if head.y >= GAME_HEIGHT - 1 {
                    self.game_over = true;
                    return false;
                }
                Point { x: head.x, y: head.y + 1 }
            }
            Direction::Left => {
                if head.x == 0 {
                    self.game_over = true;
                    return false;
                }
                Point { x: head.x - 1, y: head.y }
            }
            Direction::Right => {
                if head.x >= GAME_WIDTH - 1 {
                    self.game_over = true;
                    return false;
                }
                Point { x: head.x + 1, y: head.y }
            }
        };

        for i in 0..self.snake_len {
            if self.snake[i].x == new_head.x && self.snake[i].y == new_head.y {
                self.game_over = true;
                return false;
            }
        }

        let ate_food = new_head.x == self.food.x && new_head.y == self.food.y;

        if ate_food {
            if self.snake_len < MAX_SNAKE_LEN {
                self.snake_len += 1;
            }
            self.score += 10;
            self.spawn_food();
        }

        for i in (1..self.snake_len).rev() {
            self.snake[i] = self.snake[i - 1];
        }
        self.snake[0] = new_head;

        true
    }

    pub fn run(&mut self, writer: &mut Writer) {
        idt::flush_buffer();
        
        writer.clear();
        
        unsafe {
            let vga = VGA_BUFFER as *mut u8;
            let instructions = b"WASD to move, Q to quit";
            let start_x = 50;
            for (i, &byte) in instructions.iter().enumerate() {
                let offset = (start_x + i) * 2;
                *vga.add(offset) = byte;
                *vga.add(offset + 1) = 0x07;
            }
        }
        
        self.draw_box();
        self.draw();
        
        unsafe {
            let vga = VGA_BUFFER as *mut u8;
            let start_str = b"Press WASD to start!";
            let start_offset = ((GAME_OFFSET_Y + GAME_HEIGHT / 2) * 80 + GAME_OFFSET_X + 10) * 2;
            for (i, &byte) in start_str.iter().enumerate() {
                *vga.add(start_offset + i * 2) = byte;
                *vga.add(start_offset + i * 2 + 1) = 0x0E;
            }
        }
        
        let mut last_tick: u32 = idt::get_ticks();
        let game_speed: u32 = 10;
        
        loop {
            idt::wait_for_interrupt();
            
            while let Some(scancode) = idt::get_scancode() {
                if scancode & 0x80 != 0 {
                    continue;
                }
                
                match scancode {
                    0x11 => { // W
                        if self.direction != Direction::Down {
                            self.direction = Direction::Up;
                            self.started = true;
                        }
                    }
                    0x1F => { // S
                        if self.direction != Direction::Up {
                            self.direction = Direction::Down;
                            self.started = true;
                        }
                    }
                    0x1E => { // A
                        if self.direction != Direction::Right {
                            self.direction = Direction::Left;
                            self.started = true;
                        }
                    }
                    0x20 => { // D
                        if self.direction != Direction::Left {
                            self.direction = Direction::Right;
                            self.started = true;
                        }
                    }
                    0x10 => { // Q
                        return;
                    }
                    _ => {}
                }
            }
            
            if !self.started {
                continue;
            }
            
            let current_tick = idt::get_ticks();
            if current_tick.wrapping_sub(last_tick) >= game_speed {
                last_tick = current_tick;
                
                self.clear_game_area();
                
                if !self.update() {
                    self.draw();
                    unsafe {
                        let vga = VGA_BUFFER as *mut u8;
                        let game_over_str = b"GAME OVER! Press Q to exit";
                        let start_offset = ((GAME_OFFSET_Y + GAME_HEIGHT / 2) * 80 + GAME_OFFSET_X + 5) * 2;
                        for (i, &byte) in game_over_str.iter().enumerate() {
                            *vga.add(start_offset + i * 2) = byte;
                            *vga.add(start_offset + i * 2 + 1) = 0x4F;
                        }
                    }
                    
                    loop {
                        idt::wait_for_interrupt();
                        if let Some(scancode) = idt::get_scancode() {
                            if scancode == 0x10 { // Q
                                return;
                            }
                        }
                    }
                }
                
                self.draw();
            }
        }
    }
}

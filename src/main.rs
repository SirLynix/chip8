#![allow(unused_variables)]
mod chip8;

use chip8::state::{Chip8State, GRID_HEIGHT, GRID_WIDTH};
use minifb::{Key, Window, WindowOptions};
use std::env;
use std::fs;

const CELL_SIZE: usize = 10;

const WIDTH: usize = CELL_SIZE * GRID_WIDTH;
const HEIGHT: usize = CELL_SIZE * GRID_HEIGHT;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        println!("Usage: chip8 <rom-path>");
        return;
    }

    let rom_name = &args[1];
    let content = match fs::read(rom_name) {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to open file: {}", e);
            return;
        }
    };

    let content_len = content.len();
    let mut state = Chip8State::new(content);

    for i in 1..content_len {
        state.tick();
    }

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut grid = vec![false; GRID_WIDTH * GRID_HEIGHT];
    grid[42] = true;

    let mut window = Window::new(
        "Chip-8 Emulator - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for (index, cell) in buffer.iter_mut().enumerate() {
            let x = index % WIDTH;
            let y = index / WIDTH;

            let cell_x = x / CELL_SIZE;
            let cell_y = y / CELL_SIZE;

            *cell = if grid[cell_y * GRID_WIDTH + cell_x] {
                0xFFFFFF
            } else {
                0
            };
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer).unwrap();
    }
}

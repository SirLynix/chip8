#![allow(unused_variables)]
mod chip8;

#[macro_use]
extern crate num_derive;

use chip8::keys::Key as Chip8Key;
use chip8::state::{Chip8State, GRID_HEIGHT, GRID_WIDTH};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::cell::RefCell;
use std::env;
use std::fs;
use std::rc::Rc;
use std::time::SystemTime;

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

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let window = Rc::new(RefCell::from(
        Window::new(
            "Chip-8 Emulator - ESC to exit",
            WIDTH,
            HEIGHT,
            WindowOptions::default(),
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        }),
    ));

    let mut update_timer = SystemTime::now();

    let new_window_ref = Rc::clone(&window);

    state.set_key_callback(Box::new(move |key| -> bool {
        match key {
            Chip8Key::Key0 => new_window_ref.borrow().is_key_down(Key::NumPad0),
            Chip8Key::Key1 => new_window_ref.borrow().is_key_down(Key::NumPad1),
            Chip8Key::Key2 => new_window_ref.borrow().is_key_down(Key::NumPad2),
            Chip8Key::Key3 => new_window_ref.borrow().is_key_down(Key::NumPad3),
            Chip8Key::Key4 => new_window_ref.borrow().is_key_down(Key::NumPad4),
            Chip8Key::Key5 => new_window_ref.borrow().is_key_down(Key::NumPad5),
            Chip8Key::Key6 => new_window_ref.borrow().is_key_down(Key::NumPad6),
            Chip8Key::Key7 => new_window_ref.borrow().is_key_down(Key::NumPad7),
            Chip8Key::Key8 => new_window_ref.borrow().is_key_down(Key::NumPad8),
            Chip8Key::Key9 => new_window_ref.borrow().is_key_down(Key::NumPad9),
            Chip8Key::KeyA => new_window_ref.borrow().is_key_down(Key::A),
            Chip8Key::KeyB => new_window_ref.borrow().is_key_down(Key::B),
            Chip8Key::KeyC => new_window_ref.borrow().is_key_down(Key::C),
            Chip8Key::KeyD => new_window_ref.borrow().is_key_down(Key::D),
            Chip8Key::KeyE => new_window_ref.borrow().is_key_down(Key::E),
            Chip8Key::KeyF => new_window_ref.borrow().is_key_down(Key::F),
        }
    }));

    while window.borrow().is_open() && !window.borrow().is_key_down(Key::Escape) {
        window.borrow().get_keys_pressed(KeyRepeat::No).map(|keys| {
            for k in keys {
                match k {
                    Key::NumPad0 => state.on_key_pressed(Chip8Key::Key0),
                    Key::NumPad1 => state.on_key_pressed(Chip8Key::Key1),
                    Key::NumPad2 => state.on_key_pressed(Chip8Key::Key2),
                    Key::NumPad3 => state.on_key_pressed(Chip8Key::Key3),
                    Key::NumPad4 => state.on_key_pressed(Chip8Key::Key4),
                    Key::NumPad5 => state.on_key_pressed(Chip8Key::Key5),
                    Key::NumPad6 => state.on_key_pressed(Chip8Key::Key6),
                    Key::NumPad7 => state.on_key_pressed(Chip8Key::Key7),
                    Key::NumPad8 => state.on_key_pressed(Chip8Key::Key8),
                    Key::NumPad9 => state.on_key_pressed(Chip8Key::Key9),
                    Key::A => state.on_key_pressed(Chip8Key::KeyA),
                    Key::B => state.on_key_pressed(Chip8Key::KeyB),
                    Key::C => state.on_key_pressed(Chip8Key::KeyC),
                    Key::D => state.on_key_pressed(Chip8Key::KeyD),
                    Key::E => state.on_key_pressed(Chip8Key::KeyE),
                    Key::F => state.on_key_pressed(Chip8Key::KeyF),
                    _ => (),
                }
            }
        });

        match update_timer.elapsed() {
            Ok(d) => {
                if d.as_millis() >= 1000 / 720 {
                    state.tick();
                    update_timer = SystemTime::now();
                }
            }
            Err(err) => {
                println!("An error occurred: {}", err);
            }
        }

        if state.has_drawn() {
            for (index, cell) in buffer.iter_mut().enumerate() {
                let x = index % WIDTH;
                let y = index / WIDTH;

                let cell_x = x / CELL_SIZE;
                let cell_y = y / CELL_SIZE;

                *cell = if state.grid[cell_y * GRID_WIDTH + cell_x] {
                    0xFFFFFF
                } else {
                    0
                };
            }

            window.borrow_mut().update_with_buffer(&buffer).unwrap();
        } else {
            window.borrow_mut().update();
        }
    }
}

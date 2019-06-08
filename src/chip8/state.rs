use rand::Rng;

pub const GRID_WIDTH: usize = 64;
pub const GRID_HEIGHT: usize = 32;

const CHIP8_MEMORY: usize = 4096;
const CHIP8_PROGRAM_START: usize = 512;

use super::keys::Key;
use super::opcodes::Opcode;

use libc::{c_int, c_uint};
use std::thread;
use std::time::SystemTime;

extern crate num;

#[link(name = "Kernel32")]
extern "C" {
    fn Beep(frequency: c_uint, duration: c_uint) -> c_int;
}

fn beep(duration: u16) {
    thread::spawn(move || unsafe {
        Beep(800, duration as c_uint);
    });
}

static CHIP8_FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8State {
    delay_clock: SystemTime,
    delay_timer: u8,
    draw_flag: bool,
    index_register: u16,
    pub grid: Vec<bool>, // Temp public for tests
    key_pressed: Option<Box<Fn(Key) -> bool>>,
    memory: [u8; CHIP8_MEMORY],
    program_counter: usize,
    registers: [u8; 16],
    stack: Vec<u16>,
    waiting_for_key: Option<u8>,
}

impl Chip8State {
    pub fn new(source: Vec<u8>) -> Chip8State {
        let mut memory: [u8; CHIP8_MEMORY] = [0; CHIP8_MEMORY];

        // Copy font set into "interpreter memory"
        for i in 0..CHIP8_FONTSET.len() {
            memory[i] = CHIP8_FONTSET[i];
        }

        for i in 0..source.len() {
            memory[i + CHIP8_PROGRAM_START] = source[i];
        }

        Chip8State {
            delay_clock: SystemTime::now(),
            delay_timer: 0,
            draw_flag: false,
            index_register: 0,
            grid: vec![false; GRID_WIDTH * GRID_HEIGHT],
            key_pressed: None,
            memory: memory,
            program_counter: CHIP8_PROGRAM_START,
            registers: [0; 16],
            stack: vec![0u16, 0],
            waiting_for_key: None,
        }
    }

    fn decode_next_instruction(&self) -> Opcode {
        let opcode = (self.memory[self.program_counter] as u16) << 8
            | (self.memory[self.program_counter + 1] as u16);

        let opcode = match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => Opcode::Clear,
                0x00EE => Opcode::Return,
                _ => Opcode::CallRca {
                    address: (opcode & 0x0FFF) as u16,
                },
            },
            0x1000 => Opcode::Goto {
                address: (opcode & 0x0FFF) as u16,
            },
            0x2000 => Opcode::CallSubroutine {
                address: (opcode & 0x0FFF) as u16,
            },
            0x3000 => Opcode::CondEq {
                r: ((opcode & 0x0F00) >> 8) as u8,
                value: (opcode & 0x00FF) as u8,
            },
            0x4000 => Opcode::CondNe {
                r: ((opcode & 0x0F00) >> 8) as u8,
                value: (opcode & 0x00FF) as u8,
            },
            0x5000 => Opcode::CondVxVyEq {
                r1: ((opcode & 0x0F00) >> 8) as u8,
                r2: ((opcode & 0x00F0) >> 4) as u8,
            },
            0x6000 => Opcode::Set {
                r: ((opcode & 0x0F00) >> 8) as u8,
                value: (opcode & 0x00FF) as u8,
            },
            0x7000 => Opcode::Add {
                r: ((opcode & 0x0F00) >> 8) as u8,
                value: (opcode & 0x00FF) as u8,
            },
            0x8000 => match opcode & 0x000F {
                0x0 => Opcode::Assign {
                    dst: ((opcode & 0x0F00) >> 8) as u8,
                    src: ((opcode & 0x00F0) >> 4) as u8,
                },
                0x1 => Opcode::BitOpOr {
                    r1: ((opcode & 0x0F00) >> 8) as u8,
                    r2: ((opcode & 0x00F0) >> 4) as u8,
                },
                0x2 => Opcode::BitOpAnd {
                    r1: ((opcode & 0x0F00) >> 8) as u8,
                    r2: ((opcode & 0x00F0) >> 4) as u8,
                },
                0x3 => Opcode::BitOpXor {
                    r1: ((opcode & 0x0F00) >> 8) as u8,
                    r2: ((opcode & 0x00F0) >> 4) as u8,
                },
                0x4 => Opcode::Increment {
                    r1: ((opcode & 0x0F00) >> 8) as u8,
                    r2: ((opcode & 0x00F0) >> 4) as u8,
                },
                0x5 => Opcode::Sub {
                    r1: ((opcode & 0x0F00) >> 8) as u8,
                    r2: ((opcode & 0x00F0) >> 4) as u8,
                },
                0x6 => Opcode::BitOpShiftR {
                    r: ((opcode & 0x0F00) >> 8) as u8,
                },
                0x7 => Opcode::SubVyVx {
                    r1: ((opcode & 0x0F00) >> 8) as u8,
                    r2: ((opcode & 0x00F0) >> 4) as u8,
                },
                0xE => Opcode::BitOpShiftL {
                    r: ((opcode & 0x0F00) >> 8) as u8,
                },
                _ => Opcode::Invalid,
            },
            0x9000 => Opcode::CondVxVyNe {
                r1: ((opcode & 0x0F00) >> 8) as u8,
                r2: ((opcode & 0x00F0) >> 4) as u8,
            },
            0xA000 => Opcode::SetAddress {
                value: (opcode & 0x0FFF) as u16,
            },
            0xB000 => Opcode::Jump {
                offset: (opcode & 0x0FFF) as u16,
            },
            0xC000 => Opcode::SetRand {
                r: ((opcode & 0x0F00) >> 8) as u8,
                mask: (opcode & 0x00FF) as u8,
            },
            0xD000 => Opcode::DrawSprite {
                rx: ((opcode & 0x0F00) >> 8) as u8,
                ry: ((opcode & 0x00F0) >> 4) as u8,
                n: (opcode & 0x000F) as u8,
            },
            0xE000 => match opcode & 0xF0FF {
                0xE09E => Opcode::CondKeyPressed {
                    r: ((opcode & 0x0F00) >> 8) as u8,
                },
                0xE0A1 => Opcode::CondKeyReleased {
                    r: ((opcode & 0x0F00) >> 8) as u8,
                },
                _ => Opcode::Invalid,
            },
            0xF000 => match opcode & 0xF0FF {
                0xF007 => Opcode::GetDelayTimer {
                    r: ((opcode & 0x0F00) >> 8) as u8,
                },
                0xF00A => Opcode::WaitKeyPressed {
                    r: ((opcode & 0x0F00) >> 8) as u8,
                },
                0xF015 => Opcode::SetDelayTimer {
                    r: ((opcode & 0x0F00) >> 8) as u8,
                },
                0xF018 => Opcode::SetSoundTimer {
                    r: ((opcode & 0x0F00) >> 8) as u8,
                },
                0xF01E => Opcode::AddAddress {
                    r: ((opcode & 0x0F00) >> 8) as u8,
                },
                0xF029 => Opcode::SetSprite {
                    r: ((opcode & 0x0F00) >> 8) as u8,
                },
                0xF033 => Opcode::SetBCD {
                    r: ((opcode & 0x0F00) >> 8) as u8,
                },
                0xF055 => Opcode::StoreRegisters {
                    r: ((opcode & 0x0F00) >> 8) as u8,
                },
                0xF065 => Opcode::LoadRegisters {
                    r: ((opcode & 0x0F00) >> 8) as u8,
                },
                _ => Opcode::Invalid,
            },
            _ => Opcode::Invalid,
        };

        opcode
    }

    fn execute(&mut self, opcode: Opcode) {
        if let Some(_) = self.waiting_for_key {
            return;
        }

        self.draw_flag = false;

        //println!("{}: {:?}", self.program_counter, opcode);

        self.program_counter += match opcode {
            Opcode::Invalid => {
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                0
            }
            Opcode::Add { r, value } => {
                self.registers[r as usize] = self.registers[r as usize].overflowing_add(value).0;
                2
            }
            Opcode::AddAddress { r } => {
                self.index_register += self.registers[r as usize] as u16;
                2
            }
            Opcode::Assign { dst, src } => {
                self.registers[dst as usize] = self.registers[src as usize];
                2
            }
            Opcode::BitOpAnd { r1, r2 } => {
                self.registers[r1 as usize] =
                    self.registers[r1 as usize] & self.registers[r2 as usize];
                2
            }
            Opcode::BitOpOr { r1, r2 } => {
                self.registers[r1 as usize] =
                    self.registers[r1 as usize] | self.registers[r2 as usize];
                2
            }
            Opcode::BitOpXor { r1, r2 } => {
                self.registers[r1 as usize] =
                    self.registers[r1 as usize] ^ self.registers[r2 as usize];
                2
            }
            Opcode::BitOpShiftL { r } => {
                let x = self.registers[r as usize];

                self.registers[15] = (x & 128) >> 7;
                self.registers[r as usize] <<= 1;
                2
            }
            Opcode::BitOpShiftR { r } => {
                let x = self.registers[r as usize];

                self.registers[15] = x & 1;
                self.registers[r as usize] >>= 1;
                2
            }
            Opcode::CallSubroutine { address } => {
                self.stack.push(self.program_counter as u16);
                self.program_counter = address as usize;
                0
            }
            Opcode::Clear => {
                self.grid.clear();
                self.grid.resize(GRID_WIDTH * GRID_HEIGHT, false);
                2
            }
            Opcode::CondEq { r, value } => {
                if self.registers[r as usize] == value {
                    4
                } else {
                    2
                }
            }
            Opcode::CondKeyPressed { r } => {
                let key = self.registers[r as usize];

                if let Some(callback) = &self.key_pressed {
                    if callback(num::FromPrimitive::from_u8(key).unwrap()) {
                        4
                    } else {
                        2
                    }
                } else {
                    2
                }
            }
            Opcode::CondKeyReleased { r } => {
                let key = self.registers[r as usize];

                if let Some(callback) = &self.key_pressed {
                    if callback(num::FromPrimitive::from_u8(key).unwrap()) {
                        2
                    } else {
                        4
                    }
                } else {
                    4
                }
            }
            Opcode::CondNe { r, value } => {
                if self.registers[r as usize] != value {
                    4
                } else {
                    2
                }
            }
            Opcode::CondVxVyEq { r1, r2 } => {
                if self.registers[r1 as usize] == self.registers[r2 as usize] {
                    4
                } else {
                    2
                }
            }
            Opcode::CondVxVyNe { r1, r2 } => {
                if self.registers[r1 as usize] != self.registers[r2 as usize] {
                    4
                } else {
                    2
                }
            }
            Opcode::DrawSprite { rx, ry, n } => {
                let origin_x = self.registers[rx as usize];
                let origin_y = self.registers[ry as usize];
                self.registers[15] = 0;

                for y in 0..n {
                    let pixel = self.memory[(self.index_register + y as u16) as usize];
                    for x in 0..8 {
                        if pixel & (0x80 >> x) != 0 {
                            let cell_x = (origin_x.wrapping_add(x) as usize) % GRID_WIDTH;
                            let cell_y = (origin_y.wrapping_add(y) as usize) % GRID_HEIGHT;

                            let cell_index = GRID_WIDTH * cell_y + cell_x;

                            if self.grid[cell_index] {
                                self.grid[cell_index] = false;
                                self.registers[15] = 1;
                            } else {
                                self.grid[cell_index] = true;
                            }
                        }
                    }
                }

                self.draw_flag = true;

                2
            }
            Opcode::GetDelayTimer { r } => {
                self.registers[r as usize] = self.delay_timer;
                2
            }
            Opcode::Goto { address } => {
                self.program_counter = address as usize;
                0
            }
            Opcode::Increment { r1, r2 } => {
                let result =
                    self.registers[r1 as usize].overflowing_add(self.registers[r2 as usize]);

                self.registers[r1 as usize] = result.0;
                self.registers[15] = result.1 as u8;
                2
            }
            Opcode::LoadRegisters { r } => {
                for i in 0..((r + 1) as u16) {
                    self.registers[i as usize] = self.memory[(self.index_register + i) as usize];
                }

                2
            }
            Opcode::Return => {
                self.program_counter = self.stack.pop().unwrap() as usize;
                2
            }
            Opcode::Set { r, value } => {
                self.registers[r as usize] = value;
                2
            }
            Opcode::SetAddress { value } => {
                self.index_register = value;
                2
            }
            Opcode::SetBCD { r } => {
                let register_value = self.registers[r as usize];

                let memory_index = self.index_register as usize;

                self.memory[memory_index + 0] = register_value / 100;
                self.memory[memory_index + 1] = (register_value % 100) / 10;
                self.memory[memory_index + 2] = register_value % 10;

                2
            }
            Opcode::SetDelayTimer { r } => {
                self.delay_timer = self.registers[r as usize];
                2
            }
            Opcode::SetRand { r, mask } => {
                self.registers[r as usize] = rand::thread_rng().gen::<u8>() & mask;
                println!("Rand value: {}", self.registers[r as usize]);
                2
            }
            Opcode::SetSprite { r } => {
                let digit = self.registers[r as usize];
                self.index_register = digit as u16 * 5u16;
                2
            }
            Opcode::SetSoundTimer { r } => {
                let sound_time = self.registers[r as usize] as u16 * 1000 / 60;
                println!("Sound time: {}", sound_time);
                beep(sound_time);
                2
            }
            Opcode::Sub { r1, r2 } => {
                let x = self.registers[r1 as usize];
                let y = self.registers[r2 as usize];

                self.registers[15] = if x > y { 1 } else { 0 };

                self.registers[r1 as usize] = x.wrapping_sub(y);

                2
            }
            Opcode::SubVyVx { r1, r2 } => {
                let x = self.registers[r1 as usize];
                let y = self.registers[r2 as usize];

                self.registers[15] = if y > x { 1 } else { 0 };

                self.registers[r1 as usize] = y.wrapping_sub(x);

                2
            }
            Opcode::StoreRegisters { r } => {
                for i in 0..((r + 1) as u16) {
                    self.memory[(self.index_register + i) as usize] = self.registers[i as usize];
                }

                2
            }
            Opcode::WaitKeyPressed { r } => {
                self.waiting_for_key = Some(r);
                2
            }
            _ => {
                println!("{}: unhandled opcode {:?}", self.program_counter, opcode);
                2
            }
        } as usize;
    }

    pub fn has_drawn(&self) -> bool {
        self.draw_flag
    }

    pub fn on_key_pressed(&mut self, key: Key) {
        if let Some(register) = self.waiting_for_key {
            self.registers[register as usize] = key as u8;
            self.waiting_for_key = None;
        }
    }

    pub fn set_key_callback(&mut self, callback: Box<Fn(Key) -> bool>) {
        self.key_pressed = Some(callback);
    }

    pub fn tick(&mut self) {
        if self.delay_timer > 0 {
            match self.delay_clock.elapsed() {
                Ok(d) => {
                    if d.as_millis() >= 1000 / 60 {
                        self.delay_timer -= 1;
                        self.delay_clock = SystemTime::now();
                    }
                }
                Err(err) => {
                    println!("An error occurred: {}", err);
                }
            }
        }
        let opcode = self.decode_next_instruction();
        self.execute(opcode);
    }
}

pub const GRID_WIDTH: usize = 64;
pub const GRID_HEIGHT: usize = 32;

const CHIP8_MEMORY: usize = 4096;
const CHIP8_PROGRAM_START: usize = 512;

use super::opcodes::Opcode;

pub struct Chip8State {
    delay_timer: u16,
    index_register: u16,
    pub grid: Vec<bool>, // Temp public for tests
    memory: [u8; CHIP8_MEMORY],
    program_counter: usize,
    registers: [u8; 16],
    sound_timer: u16,
    stack: Vec<u16>,
}

impl Chip8State {
    pub fn new(source: Vec<u8>) -> Chip8State {
        let mut memory: [u8; CHIP8_MEMORY] = [0; CHIP8_MEMORY];

        for i in 0..source.len() {
            memory[i + CHIP8_PROGRAM_START] = source[i];
        }

        Chip8State {
            delay_timer: 0,
            index_register: 0,
            grid: vec![false; GRID_WIDTH * GRID_HEIGHT],
            memory: memory,
            program_counter: CHIP8_PROGRAM_START,
            registers: [0; 16],
            sound_timer: 0,
            stack: vec![0u16, 0],
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
                    r: (opcode & 0x0F00) as u8,
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
                origin_x: ((opcode & 0x0F00) >> 8) as u8,
                origin_y: ((opcode & 0x00F0) >> 4) as u8,
                n: (opcode & 0x000F) as u8,
            },
            0xE000 => match opcode & 0xF0FF {
                0xE09E => Opcode::CondKeyPressed {
                    key: ((opcode & 0x0F00) >> 8) as u8,
                },
                0xE0A1 => Opcode::CondKeyReleased {
                    key: ((opcode & 0x0F00) >> 8) as u8,
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
                    value: ((opcode & 0x0F00) >> 8) as u8,
                },
                0xF018 => Opcode::SetSoundTimer {
                    value: ((opcode & 0x0F00) >> 8) as u8,
                },
                0xF01E => Opcode::AddAddress {
                    offset: ((opcode & 0x0F00) >> 8) as u8,
                },
                0xF029 => Opcode::SetSprite {
                    digit: ((opcode & 0x0F00) >> 8) as u8,
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
        println!("{}: {:?}", self.program_counter, opcode);

        self.program_counter += match opcode {
            Opcode::Invalid => {
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                0
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
            Opcode::DrawSprite {
                origin_x,
                origin_y,
                n,
            } => {
                for y in 0..n {
                    let pixel = self.memory[(self.index_register + y as u16) as usize];
                    for x in 0..8 {
                        if pixel & (0x80 >> x) != 0 {
                            //TODO: Handle xor + coordinates wrap
                            self.grid
                                [GRID_WIDTH * (origin_y + y) as usize + (origin_x + x) as usize] =
                                true;
                        }
                    }
                }
                2
            }
            Opcode::Return => {
                self.program_counter = self.stack.pop().unwrap() as usize;
                0
            }
            Opcode::Set { r, value } => {
                self.registers[r as usize] = value;
                2
            }
            Opcode::SetAddress { value } => {
                self.index_register = value;
                2
            }
            _ => 2,
        } as usize;
    }

    pub fn tick(&mut self) {
        let opcode = self.decode_next_instruction();
        self.execute(opcode);
    }
}

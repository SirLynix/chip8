pub const GRID_WIDTH: usize = 64;
pub const GRID_HEIGHT: usize = 32;

use super::opcodes::Opcode;

pub struct Chip8State {
    address_register: usize,
    delay_timer: u16,
    grid: Vec<bool>,
    registers: [u8; 16],
    sound_timer: u16,
    source: Vec<u8>,
    stack: Vec<u16>,
}

impl Chip8State {
    pub fn new(source: Vec<u8>) -> Chip8State {
        Chip8State {
            address_register: 0,
            delay_timer: 0,
            grid: vec![false; GRID_WIDTH * GRID_HEIGHT],
            registers: [0; 16],
            sound_timer: 0,
            source: source,
            stack: vec![0u16, 0],
        }
    }

    fn decode_next_instruction(&self) {
        let opcode = (self.source[self.address_register] as u16) << 8
            | (self.source[self.address_register + 1] as u16);

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
            0x2000 => Opcode::CallSubroutine,
            0x3000 => Opcode::CondEq,
            0x4000 => Opcode::CondNe,
            0x5000 => Opcode::CondVxVyEq,
            0x6000 => Opcode::Set {
                r: (opcode & 0x0F00 >> 8) as u8,
                value: (opcode & 0x00FF) as u8,
            },
            0x7000 => Opcode::Add,
            0x8000 => match opcode & 0x000F {
                0x0 => Opcode::Assign,
                0x1 => Opcode::BitOpOr,
                0x2 => Opcode::BitOpAnd,
                0x3 => Opcode::BitOpXor,
                0x4 => Opcode::Increment,
                0x5 => Opcode::Sub,
                0x6 => Opcode::BitOpShiftR,
                0x7 => Opcode::SubVyVx,
                0xE => Opcode::BitOpShiftL,
                _ => Opcode::Invalid,
            },
            0x9000 => Opcode::CondVxVyNe,
            0xA000 => Opcode::SetAddress,
            0xB000 => Opcode::Jump,
            0xC000 => Opcode::SetRand,
            0xD000 => Opcode::DrawSprite,
            0xE000 => match opcode & 0xF0FF {
                0xE09E => Opcode::CondKeyPressed,
                0xE0A1 => Opcode::CondKeyReleased,
                _ => Opcode::Invalid,
            },
            0xF000 => match opcode & 0xF0FF {
                0xF007 => Opcode::GetDelayTimer,
                0xF00A => Opcode::WaitKeyPressed,
                0xF015 => Opcode::SetDelayTimer,
                0xF018 => Opcode::SetSoundTimer,
                0xF01E => Opcode::AddAddress,
                0xF029 => Opcode::SetSprite,
                0xF033 => Opcode::SetBCD,
                0xF055 => Opcode::StoreRegisters,
                0xF065 => Opcode::LoadRegisters,
                _ => Opcode::Invalid,
            },
            _ => Opcode::Invalid,
        };

        match opcode {
            Opcode::Invalid => println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!"),
            _ => (),
        }

        println!("{}: {:?}", self.address_register, opcode);
    }

    pub fn tick(&mut self) {
        self.decode_next_instruction();
        self.address_register += 2;
    }
}

#[derive(Debug)]
pub enum Opcode {
    Invalid,

    Add { r: u8, value: u8 },             // ADD Vx, byte - 7XNN
    AddAddress { r: u8 },                 // ADD I, Vx - FX1E
    Assign { dst: u8, src: u8 },          // LD Vx, Vy - 8XY0
    BitOpAnd { r1: u8, r2: u8 },          // AND Vx, Vy - 8XY2
    BitOpOr { r1: u8, r2: u8 },           // OR Vx, Vy - 8XY1
    BitOpShiftL { r: u8 },                // SHL Vx {, Vy} - 8XYE
    BitOpShiftR { r: u8 },                // SHR Vx {, Vy} - 8XY6
    BitOpXor { r1: u8, r2: u8 },          // XOR Vx, Vy - 8XY3
    CallRca { address: u16 },             // SYS addr - 0NNN
    CallSubroutine { address: u16 },      // CALL addr - 2NNN
    Clear,                                // CLS - 00E0
    CondEq { r: u8, value: u8 },          // SE Vx, byte - 3XNN
    CondKeyPressed { r: u8 },             // SKP - EX9E
    CondKeyReleased { r: u8 },            // SKNP - EXA1
    CondNe { r: u8, value: u8 },          // SNE Vx, byte - 4XNN
    CondVxVyEq { r1: u8, r2: u8 },        // SE Vx, Vy - 5XY0
    CondVxVyNe { r1: u8, r2: u8 },        // SNE Vx, Vy - 9XY0
    DrawSprite { rx: u8, ry: u8, n: u8 }, // DRW Vx, Vy, nibble - DXYN
    GetDelayTimer { r: u8 },              // LD Vx, DT - FX07
    Goto { address: u16 },                // JP addr - 1NNN
    Increment { r1: u8, r2: u8 },         // ADD Vx, Vy - 8XY4
    Jump { offset: u16 },                 // JP V0, addr - BNNN
    LoadRegisters { r: u8 },              // LD Vx, [I] - FX65
    Return,                               // RET - 00EE
    Set { r: u8, value: u8 },             // LD Vx, byte - 6XNN
    SetAddress { value: u16 },            // LD I, addr - ANNN
    SetBCD { r: u8 },                     // LD B, Vx - FX33
    SetDelayTimer { r: u8 },              // LD DT, Vx - FX15
    SetRand { r: u8, mask: u8 },          // RND Vx, byte - CXNN
    SetSoundTimer { r: u8 },              // LD ST, Vx - FX18
    SetSprite { r: u8 },                  // LD F, Vx - FX29
    StoreRegisters { r: u8 },             // LD [I], Vx - FX55
    Sub { r1: u8, r2: u8 },               // SUB Vx, Vy - 8XY5
    SubVyVx { r1: u8, r2: u8 },           // SUBN Vx, Vy - 8XY7
    WaitKeyPressed { r: u8 },             // LD Vx, K - FX0A
}

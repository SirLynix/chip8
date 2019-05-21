#[derive(Debug)]
pub enum Opcode {
    Invalid,

    Add,                      // 7XNN
    AddAddress,               // FX1E
    Assign,                   // 8XY0
    BitOpAnd,                 // 8XY2
    BitOpOr,                  // 8XY1
    BitOpShiftL,              // 8XYE
    BitOpShiftR,              // 8XY6
    BitOpXor,                 // 8XY3
    CallRca { address: u16 }, // 0NNN
    CallSubroutine,           // 2NNN
    Clear,                    // 00E0
    CondEq,                   // 3XNN
    CondKeyPressed,           // EX9E
    CondKeyReleased,          // EXA1
    CondNe,                   // 4XNN
    CondVxVyEq,               // 5XY0
    CondVxVyNe,               // 9XY0
    DrawSprite,               // DXYN
    GetDelayTimer,            // FX07
    Goto { address: u16 },    // 1NNN
    Increment,                // 8XY4
    Jump,                     // BNNN
    LoadRegisters,            // FX65
    Return,                   // 00EE
    Set { r: u8, value: u8 }, // 6XNN
    SetAddress,               // ANNN
    SetBCD,                   // FX33
    SetDelayTimer,            // FX15
    SetRand,                  // CXNN
    SetSoundTimer,            // FX18
    SetSprite,                // FX29
    StoreRegisters,           // FX55
    Sub,                      // 8XY5
    SubVyVx,                  // 8XY7
    WaitKeyPressed,           // FX0A
}

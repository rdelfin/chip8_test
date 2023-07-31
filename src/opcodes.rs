use crate::emulator::Chip8State;

/// Data extracted from the 16-bit opcode. Uniform across all opcodes (though not used by all).
struct OpCodeData {
    /// Second nibble. ALWAYS used for register indexes
    pub x: u8,
    /// Third nibble, also ALWAYS used to index registers
    pub y: u8,
    /// Fourth nibble
    pub n: u8,
    /// Second byte
    pub nn: u8,
    /// Second, third, and fourth nibble
    pub nnn: u8,
}

/// Implemented by any struct that can read a specific Chip8 opcode
trait OpCodeReader {
    /// This is the value identifying the opcode. It'll be matched against the mask bellow so be
    /// sure to set any valiable bits to 0
    fn opcode_val(&self) -> u16;

    /// Mask of the opcode prefix
    fn opcode_mask(&self) -> u16;

    /// Use this to actually process a chip 8 opcode from a given CPU state and decoded
    /// instruction. Note we will have incremented PC  by 2 bytes by the time this is called
    fn execute(&self, state: &mut Chip8State);
}

#[derive(Debug, Default, Clone)]
pub struct ClearScreen;

impl OpCodeReader for ClearScreen {
    fn opcode_val(&self) -> u16 {
        0x00e0
    }

    fn opcode_mask(&self) -> u16 {
        0xffff
    }

    fn execute(&self, state: &mut Chip8State) {}
}

#[derive(Debug, Default, Clone)]
pub struct Jump;

impl OpCodeReader for Jump {
    fn opcode_val(&self) -> u16 {
        0x1000
    }

    fn opcode_mask(&self) -> u16 {
        0xf000
    }

    fn execute(&self, state: &mut Chip8State) {}
}

#[derive(Debug, Default, Clone)]
pub struct SetRegister;

impl OpCodeReader for SetRegister {
    fn opcode_val(&self) -> u16 {
        0x6000
    }

    fn opcode_mask(&self) -> u16 {
        0xf000
    }

    fn execute(&self, state: &mut Chip8State) {}
}

#[derive(Debug, Default, Clone)]
pub struct AddRegister;

impl OpCodeReader for AddRegister {
    fn opcode_val(&self) -> u16 {
        0x7000
    }

    fn opcode_mask(&self) -> u16 {
        0xf000
    }

    fn execute(&self, state: &mut Chip8State) {}
}

#[derive(Debug, Default, Clone)]
pub struct SetIndexRegister;

impl OpCodeReader for SetIndexRegister {
    fn opcode_val(&self) -> u16 {
        0xA000
    }

    fn opcode_mask(&self) -> u16 {
        0xf000
    }

    fn execute(&self, state: &mut Chip8State) {}
}

#[derive(Debug, Default, Clone)]
pub struct DisplayDraw;

impl OpCodeReader for DisplayDraw {
    fn opcode_val(&self) -> u16 {
        0xD000
    }

    fn opcode_mask(&self) -> u16 {
        0xf000
    }

    fn execute(&self, state: &mut Chip8State) {}
}

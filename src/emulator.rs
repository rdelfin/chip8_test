use crate::{
    display::Display,
    font::Chip8Font,
    opcodes::{self, OpCodeData, OpCodeReader},
};
use byteorder::{BigEndian, ByteOrder};
use std::{collections::VecDeque, fmt};

pub struct EmulatedChip8 {
    state: Chip8State,
    supported_instructions: Vec<Box<dyn OpCodeReader>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chip8State {
    pub memory: [u8; 4096],
    pub display: Display,
    pub pc: Address,
    pub stack: VecDeque<Address>,
    pub index_register: Address,
    pub delay_timer: Register,
    pub sound_timer: Register,
    pub gp_registers: [Register; 16],
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("the opcode {0:#06x} is unsupported")]
    UnsupportedOpcode(u16),
}

pub type Result<T = (), E = Error> = std::result::Result<T, E>;

impl EmulatedChip8 {
    /// Creates a new, empty, uninitialised emulated chip 8
    /// Usually you'd call this, followed by [`EmulatedChip8::write_font`],
    /// [`EmulatedChip8::load_program`], and then regularly call [`EmulatedChip8::step`].
    pub fn new() -> EmulatedChip8 {
        EmulatedChip8 {
            state: Chip8State::new(),
            supported_instructions: vec![
                Box::new(opcodes::ClearScreen),
                Box::new(opcodes::Jump),
                Box::new(opcodes::SetRegister),
                Box::new(opcodes::AddRegister),
                Box::new(opcodes::SetIndexRegister),
                Box::new(opcodes::DisplayDraw),
            ],
        }
    }

    /// Use this to write a font to the appropriate location in memory.
    /// # Arguments
    /// * `font` - The font data to load onto memory
    pub fn write_font(&mut self, font: &Chip8Font) {
        font.write(&mut self.state);
    }

    /// Runs a single step on the CPU. In this case, this practically will execute a full
    /// fetch-decode-execute loop on the emulated CPU.
    pub fn step(&mut self) -> Result {
        let opcode_bytes = self.fetch();
        let opcode_data = self.decode(opcode_bytes);
        self.execute(opcode_data)
    }

    /// Returns the underlying chip8 state for inspection, use, or display.
    pub fn get_state(&self) -> &Chip8State {
        &self.state
    }

    fn fetch(&mut self) -> u16 {
        let opcode_bytes = BigEndian::read_u16(&self.state.memory[self.state.pc.0.into()..]);
        // Always increment PC in fetch stage
        self.state.pc += 2;
        opcode_bytes
    }

    fn decode(&mut self, opcode_bytes: u16) -> OpCodeData {
        let mut bytes = [0u8, 0u8];
        BigEndian::write_u16(&mut bytes, opcode_bytes);
        let bytes_u16 = [bytes[0] as u16, bytes[1] as u16];

        OpCodeData {
            full_opcode: opcode_bytes,
            x: bytes[0] & 0x0F,
            y: (bytes[1] & 0xF0) >> 4,
            n: bytes[1] & 0x0F,
            nn: bytes[1],
            nnn: ((bytes_u16[0] & 0x000F) << 8) | bytes_u16[1],
        }
    }

    fn execute(&mut self, opcode_data: OpCodeData) -> Result<()> {
        for instruction in &self.supported_instructions {
            if opcode_data.full_opcode & instruction.opcode_mask() == instruction.opcode_val() {
                instruction.execute(&mut self.state);
                return Ok(());
            }
        }

        Err(Error::UnsupportedOpcode(opcode_data.full_opcode))
    }
}

impl fmt::Display for EmulatedChip8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.state)
    }
}

impl Chip8State {
    pub fn new() -> Chip8State {
        Chip8State {
            memory: [0; 4096],
            display: Display::default(),
            pc: Address(0),
            stack: VecDeque::new(),
            index_register: Address(0),
            delay_timer: Register(0),
            sound_timer: Register(0),
            gp_registers: [Register(0); 16],
        }
    }
}

impl fmt::Display for Chip8State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.display)?;
        writeln!(f, "Registers:")?;
        for register in self.gp_registers {
            write!(f, "{register} ")?;
        }
        writeln!(f)?;
        writeln!(f, "PC: {}    DT: {}", self.pc, self.delay_timer)?;
        writeln!(f, "IR: {}    ST: {}", self.index_register, self.sound_timer)?;
        write!(f, "Stack: [")?;
        for (idx, val) in self.stack.iter().enumerate() {
            let space = if idx == self.stack.len() - 1 { "" } else { " " };
            write!(f, "{val}{space}")?;
        }
        writeln!(f, "]")?;
        writeln!(f, "Memory:")?;
        for (idx, byte) in self.memory.iter().enumerate() {
            write!(f, "{byte:02x} ")?;
            if idx % 64 == 63 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Address(pub u16);

impl From<Address> for usize {
    fn from(address: Address) -> usize {
        address.0.into()
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:04x}", self.0)
    }
}

impl std::ops::AddAssign<u16> for Address {
    fn add_assign(&mut self, other: u16) {
        self.0 = self.0.overflowing_add(other).0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Register(pub u8);

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:02x}", self.0)
    }
}

#[cfg(test)]
mod test {
    use super::EmulatedChip8;
    use crate::opcodes::OpCodeData;

    #[test]
    fn test_decode() {
        let mut chip = EmulatedChip8::new();
        let decoded = chip.decode(0x1B3D);

        assert_eq!(
            decoded,
            OpCodeData {
                full_opcode: 0x1B3D,
                x: 0x0B,
                y: 0x03,
                n: 0x0D,
                nn: 0x3D,
                nnn: 0xB3D,
            }
        );
    }
}

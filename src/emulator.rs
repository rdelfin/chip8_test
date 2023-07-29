use crate::display::Display;
use std::{collections::VecDeque, fmt};

#[derive(Debug, Clone)]
pub struct EmulatedChip8 {
    state: Chip8State,
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
pub enum Error {}

pub type Result<T = (), E = Error> = std::result::Result<T, E>;

impl EmulatedChip8 {
    pub fn new() -> EmulatedChip8 {
        EmulatedChip8 {
            state: Chip8State::new(),
        }
    }

    pub fn step(&mut self) -> Result {
        Ok(())
    }

    pub fn get_state(&self) -> &Chip8State {
        &self.state
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

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:04x}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Register(pub u8);

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:02x}", self.0)
    }
}

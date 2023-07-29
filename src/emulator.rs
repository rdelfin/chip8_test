use crate::display::Display;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmulatedChip8 {
    memory: [u8; 4096],
    display: Display,
    pc: Address,
    stack: VecDeque<Address>,
    index_register: Address,
    delay_timer: Register,
    sound_timer: Register,
    gp_registers: [Register; 16],
}

impl EmulatedChip8 {
    pub fn new() -> EmulatedChip8 {
        EmulatedChip8 {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Address(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Register(pub u8);

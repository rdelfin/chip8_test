use crate::{
    display::Display,
    font::Chip8Font,
    opcodes::{self, OpCodeData, OpCodeReader},
    program::Program,
};
use byteorder::{BigEndian, ByteOrder};
use log::debug;
use std::{collections::VecDeque, fmt, time::Duration};

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
    pub since_last_delay_update: Duration,
    pub sound_timer: Register,
    pub since_last_sound_update: Duration,
    pub gp_registers: [Register; 16],
    pub key_state: KeyInput,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyInput {
    pub key_state: [bool; 0x10],
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
                Box::new(opcodes::SetRegisterConst),
                Box::new(opcodes::AddRegisterConst),
                Box::new(opcodes::SetIndexRegister),
                Box::new(opcodes::DisplayDraw),
                Box::new(opcodes::SubroutineCall),
                Box::new(opcodes::SubroutineReturn),
                Box::new(opcodes::SkipConstEqual),
                Box::new(opcodes::SkipConstNotEqual),
                Box::new(opcodes::SkipRegistersEqual),
                Box::new(opcodes::SkipRegistersNotEqual),
                Box::new(opcodes::SetRegisterRegister),
                Box::new(opcodes::BinaryOr),
                Box::new(opcodes::BinaryAnd),
                Box::new(opcodes::BinaryXor),
                Box::new(opcodes::AddRegisters),
                Box::new(opcodes::SubtractRegisters),
                Box::new(opcodes::SubtractRegistersReverse),
                Box::new(opcodes::ShiftRegisterRight),
                Box::new(opcodes::ShiftRegisterLeft),
                Box::new(opcodes::JumpOffset),
                Box::new(opcodes::Random),
                Box::new(opcodes::SkipIfKey),
                Box::new(opcodes::SkipIfNotKey),
                Box::new(opcodes::ReadDelayTimer),
                Box::new(opcodes::SetDelayTimer),
                Box::new(opcodes::SetSoundTimer),
                Box::new(opcodes::AddIndexRegister),
                Box::new(opcodes::GetKey),
                Box::new(opcodes::ReadFontCharacter),
                Box::new(opcodes::DecimalDecoding),
                Box::new(opcodes::StoreMemory),
                Box::new(opcodes::LoadMemory),
            ],
        }
    }

    /// Use this to write a font to the appropriate location in memory.
    /// # Arguments
    /// * `font` - The font data to load onto memory
    pub fn write_font(&mut self, font: &Chip8Font) {
        font.write(&mut self.state);
    }

    /// Use this to write a program to the appropriate location in memory.
    /// # Arguments
    /// * `program` - The program data to load onto memory
    pub fn load_program(&mut self, program: &Program) {
        program.load(&mut self.state);
    }

    /// Runs a single step on the CPU. In this case, this practically will execute a full
    /// fetch-decode-execute loop on the emulated CPU. We also expect you to provide keyboard input
    pub fn step(&mut self, key_input: KeyInput, time_delta: Duration) -> Result {
        self.state.key_state = key_input;
        self.update_timers(time_delta);
        let opcode_bytes = self.fetch();
        let opcode_data = self.decode(opcode_bytes);
        self.execute(opcode_data)
    }

    /// Returns the underlying chip8 state for inspection, use, or display.
    pub fn get_state(&self) -> &Chip8State {
        &self.state
    }

    fn update_timers(&mut self, time_delta: Duration) {
        update_timer(
            &mut self.state.delay_timer,
            &mut self.state.since_last_delay_update,
            time_delta,
        );
        update_timer(
            &mut self.state.sound_timer,
            &mut self.state.since_last_sound_update,
            time_delta,
        );
    }

    fn fetch(&mut self) -> u16 {
        let opcode_bytes = BigEndian::read_u16(&self.state.memory[self.state.pc.0.into()..]);
        // Always increment PC in fetch stage
        self.state.pc += 2;
        opcode_bytes
    }

    fn decode(&mut self, opcode_bytes: u16) -> OpCodeData {
        OpCodeData::decode(opcode_bytes)
    }

    fn execute(&mut self, opcode_data: OpCodeData) -> Result<()> {
        for instruction in &self.supported_instructions {
            if opcode_data.full_opcode & instruction.opcode_mask() == instruction.opcode_val() {
                debug!("Executing instruction {instruction:?} with opcode data {opcode_data:?}; pc: {:#x}", self.state.pc.0);
                instruction.execute(&mut self.state, opcode_data);
                return Ok(());
            }
        }

        Err(Error::UnsupportedOpcode(opcode_data.full_opcode))
    }
}

const DECREMENT_PERIOD: Duration = Duration::from_millis(17);

fn update_timer(register: &mut Register, since_last_update: &mut Duration, time_delta: Duration) {
    if register.0 > 0 {
        // Target frequency at which we reduce is 60Hz (period ~16.67ms). To do that we need to
        // consider two cases: time_delta > 16.67ms (in which case, we need to decrement multiple
        // times), and time_delta < 16.67ms (in which case we need to keep track of how much time
        // until the next time we decrement). We will handle both together by:
        // - Adding the time delta to the time since last update
        // - Remove decrement period from that new time since last update until we can no longer
        // - Store back any reminder
        // The period at which we decrement the timer is represented by `DECREMENT_PERIOD` (which
        // we round up to 17ms for simplicity)

        let mut new_since_last_update = *since_last_update + time_delta;
        while new_since_last_update > DECREMENT_PERIOD {
            if register.0 > 1 {
                register.0 -= 1;
                new_since_last_update -= DECREMENT_PERIOD;
            }
            // Special case: if we reach 1 and need to subtract again, we should just reset
            // `since_last_update` and stop
            else {
                register.0 = 0;
                *since_last_update = Duration::default();
                return;
            }
        }

        // new_since_last_update will contain the reminder of the "division" we made
        *since_last_update = new_since_last_update;
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
            since_last_delay_update: Duration::default(),
            sound_timer: Register(0),
            since_last_sound_update: Duration::default(),
            gp_registers: [Register(0); 16],
            key_state: KeyInput::default(),
        }
    }

    #[cfg(test)]
    pub fn with_display(mut self, display: Display) -> Chip8State {
        self.display = display;
        self
    }

    #[cfg(test)]
    pub fn with_pc(mut self, pc: Address) -> Chip8State {
        self.pc = pc;
        self
    }

    #[cfg(test)]
    pub fn with_stack(mut self, stack: VecDeque<Address>) -> Chip8State {
        self.stack = stack;
        self
    }

    #[cfg(test)]
    pub fn with_index_register(mut self, index: Address) -> Chip8State {
        self.index_register = index;
        self
    }

    #[cfg(test)]
    pub fn with_delay_timer(mut self, delay_timer: Register) -> Chip8State {
        self.delay_timer = delay_timer;
        self
    }

    #[cfg(test)]
    pub fn with_sound_timer(mut self, sound_timer: Register) -> Chip8State {
        self.sound_timer = sound_timer;
        self
    }

    #[cfg(test)]
    pub fn with_register(mut self, register: Register, index: u8) -> Chip8State {
        self.gp_registers[index as usize] = register;
        self
    }

    #[cfg(test)]
    pub fn with_memory_set(mut self, bytes: &[u8], start: Address) -> Chip8State {
        self.memory_set(bytes, start);
        self
    }

    #[cfg(test)]
    pub fn with_key_pressed(mut self, key: u8) -> Chip8State {
        self.key_state.key_state[usize::from(key)] = true;
        self
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        self.key_state.key_state[usize::from(key)]
    }

    pub fn memory_set(&mut self, bytes: &[u8], start: Address) {
        let byte_start = usize::from(start.0);
        let byte_end = byte_start + bytes.len();
        // If byte_end is *exactly* 0x1000 we can still write (as the end is one past the last
        // element), but if we go over that we're writing past the end
        if byte_end > 0x1000 {
            panic!("asking to write past last byte");
        }
        self.memory[byte_start..byte_end].copy_from_slice(bytes);
    }

    pub fn gp_register(&mut self, index: u8) -> &mut Register {
        &mut self.gp_registers[index as usize]
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

impl std::ops::AddAssign<u8> for Register {
    fn add_assign(&mut self, other: u8) {
        self.0 = self.0.overflowing_add(other).0
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

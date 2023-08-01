use crate::emulator::{Address, Chip8State};
use std::path::Path;

pub struct Program {
    data: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the program is too large to load onto memory")]
    ProgramTooLarge,
    #[error("could not read the ROM file: {0}")]
    CouldNotRead(#[source] std::io::Error),
}

impl Program {
    pub fn new_from_data(data: &[u8]) -> Result<Program, Error> {
        // We only accept programs of up to 2KB
        if data.len() > 2 * 1024 {
            return Err(Error::ProgramTooLarge);
        }

        Ok(Program {
            data: data.to_vec(),
        })
    }

    pub fn new_from_file<P: AsRef<Path>>(path: P) -> Result<Program, Error> {
        let data = std::fs::read(path).map_err(Error::CouldNotRead)?;
        Self::new_from_data(&data[..])
    }

    pub fn load(&self, state: &mut Chip8State) {
        let start_idx = 0x200;
        let end_idx = start_idx + self.data.len();
        state.memory[start_idx..end_idx].copy_from_slice(&self.data[..]);

        // Set PC to program start
        state.pc = Address(start_idx as u16);
    }
}

use crate::emulator::Chip8State;
use byteorder::{BigEndian, ByteOrder};

/// Data extracted from the 16-bit opcode. Uniform across all opcodes (though not used by all).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OpCodeData {
    /// Full 2 bytes of the opcode. Anything between the first nibble and the full 2 bytes can
    /// contain the ID of the opcode
    pub full_opcode: u16,
    /// Second nibble. ALWAYS used for register indexes
    pub x: u8,
    /// Third nibble, also ALWAYS used to index registers
    pub y: u8,
    /// Fourth nibble
    pub n: u8,
    /// Second byte
    pub nn: u8,
    /// Second, third, and fourth nibble
    pub nnn: u16,
}

impl OpCodeData {
    pub fn decode(opcode_bytes: u16) -> OpCodeData {
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
}

/// Implemented by any struct that can read a specific Chip8 opcode
pub trait OpCodeReader {
    /// This is the value identifying the opcode. It'll be matched against the mask bellow so be
    /// sure to set any valiable bits to 0
    fn opcode_val(&self) -> u16;

    /// Mask of the opcode prefix
    fn opcode_mask(&self) -> u16;

    /// Use this to actually process a chip 8 opcode from a given CPU state and decoded
    /// instruction. Note we will have incremented PC  by 2 bytes by the time this is called
    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData);
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

    fn execute(&self, _state: &mut Chip8State, _opcode_data: OpCodeData) {}
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

    fn execute(&self, _state: &mut Chip8State, _opcode_data: OpCodeData) {}
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

    fn execute(&self, _state: &mut Chip8State, _opcode_data: OpCodeData) {}
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

    fn execute(&self, _state: &mut Chip8State, _opcode_data: OpCodeData) {}
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

    fn execute(&self, _state: &mut Chip8State, _opcode_data: OpCodeData) {}
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

    fn execute(&self, _state: &mut Chip8State, _opcode_data: OpCodeData) {}
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        display::{Coordinates, Display},
        emulator::{Address, Register},
    };
    use expect_test::expect;

    #[test]
    fn test_decode() {
        let decoded = OpCodeData::decode(0x1B3D);

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

    #[test]
    fn test_clear_screen() {
        let cs_reader = ClearScreen;
        let expected_screen = expect![
            r#"
            .----------------------------------------------------------------.
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            .----------------------------------------------------------------."#
        ];

        for _ in 0..1 {
            let mut state = Chip8State::new();
            for col in state.display.pixels.iter_mut() {
                for pixel in col.iter_mut() {
                    *pixel = rand::random();
                }
            }
            cs_reader.execute(&mut state, OpCodeData::decode(0x00e0));
            // expected_screen.assert_eq(&state.display.to_string());
        }
    }

    #[test]
    fn test_jump() {
        let jump_reader = Jump;
        let mut state = Chip8State::new().with_pc(Address(100));
        let correct_state = state.clone().with_pc(Address(0x1de));
        jump_reader.execute(&mut state, OpCodeData::decode(0x11de));
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_set_register() {
        let sr_reader = SetRegister;
        let mut state = Chip8State::new().with_register(Register(0xef), 2);
        let mut correct_state = state.clone().with_register(Register(0x12), 2);
        sr_reader.execute(&mut state, OpCodeData::decode(0x6212));
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_add_register() {
        let ar_reader = AddRegister;
        let mut state = Chip8State::new().with_register(Register(0x43), 0x0a);
        let mut correct_state = state.clone().with_register(Register(0x7d), 0x0a);
        ar_reader.execute(&mut state, OpCodeData::decode(0x7a3a));
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_set_index_register() {
        let sir_reader = SetIndexRegister;
        let mut state = Chip8State::new().with_register(Register(0x43), 0x0a);
        let mut correct_state = state.clone().with_register(Register(0x7d), 0x0a);
        sir_reader.execute(&mut state, OpCodeData::decode(0x7a3a));
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_display_draw() {
        let d_reader = DisplayDraw;
        let display = {
            let mut display = Display::default();
            display.flip_all(Coordinates::new(52, 10), Coordinates::new(61, 19));
            display
        };
        let mut state = Chip8State::new()
            .with_display(display.clone())
            .with_index_register(Address(0x300))
            .with_register(Register(56), 2) // Stores X
            .with_register(Register(8), 3); // Stores Y

        // Sprite described below
        state.memory[0x300..0x302].copy_from_slice(&[0xF7, 0x93, 0xDF]);
        // "random" data to ensure we don't read past end
        state.memory[0x303..0x305].copy_from_slice(&[0xAB, 0x41, 0x9A]);

        let expected_screen = expect![[r#"
            .----------------------------------------------------------------.
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                    ██████████  |
            |                                                    ██████████  |
            |                                                    ██████████  |
            |                                                    ██████████  |
            |                                                    ██████████  |
            |                                                    ██████████  |
            |                                                    ██████████  |
            |                                                    ██████████  |
            |                                                    ██████████  |
            |                                                    ██████████  |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            .----------------------------------------------------------------."#]];
        expected_screen.assert_eq(&state.display.to_string());

        // Currently we have a box of 10x10 starting at 56,8
        // We will draw the following sprite:
        // ████ ███  (0b11110111 = 0xF7)
        // █  █  ██  (0b10010011 = 0x93)
        // ██ █████  (0b11011111 = 0xDF)
        //
        // We'll draw from 56,8, resulting in: (from 52,8):
        //|    ████ ███|
        //|    █  █  ██|
        //|████  █  ███|
        //|█████████   |
        //|█████████   |
        //[etc.]

        d_reader.execute(&mut state, OpCodeData::decode(0xD233));

        let after_screen = expect![[r#"
            .----------------------------------------------------------------.
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                        ████ ███|
            |                                                        █  █  ██|
            |                                                    ████  █  ███|
            |                                                    ██████████  |
            |                                                    ██████████  |
            |                                                    ██████████  |
            |                                                    ██████████  |
            |                                                    ██████████  |
            |                                                    ██████████  |
            |                                                    ██████████  |
            |                                                    ██████████  |
            |                                                    ██████████  |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            |                                                                |
            .----------------------------------------------------------------."#]];
        after_screen.assert_eq(&state.display.to_string());
    }
}

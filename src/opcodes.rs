use crate::{
    display::Coordinates,
    emulator::{Address, Chip8State, Register},
};
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
pub trait OpCodeReader: std::fmt::Debug {
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

    fn execute(&self, state: &mut Chip8State, _: OpCodeData) {
        state.display.clear();
    }
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

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        state.pc = Address(opcode_data.nnn);
    }
}

#[derive(Debug, Default, Clone)]
pub struct SetRegisterConst;

impl OpCodeReader for SetRegisterConst {
    fn opcode_val(&self) -> u16 {
        0x6000
    }

    fn opcode_mask(&self) -> u16 {
        0xf000
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        *state.gp_register(opcode_data.x) = Register(opcode_data.nn);
    }
}

#[derive(Debug, Default, Clone)]
pub struct AddRegisterConst;

impl OpCodeReader for AddRegisterConst {
    fn opcode_val(&self) -> u16 {
        0x7000
    }

    fn opcode_mask(&self) -> u16 {
        0xf000
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        *state.gp_register(opcode_data.x) += opcode_data.nn;
    }
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

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        state.index_register = Address(opcode_data.nnn);
    }
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

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        let draw_coordinates = Coordinates::new(
            state.gp_register(opcode_data.x).0,
            state.gp_register(opcode_data.y).0,
        );
        let rows: usize = opcode_data.n.into();
        let sprite_start: usize = state.index_register.into();
        let sprite_end = sprite_start + rows;
        let sprite = &state.memory[sprite_start..sprite_end];
        state.display.apply_sprite(sprite, draw_coordinates);
    }
}

#[derive(Debug, Default, Clone)]
pub struct SubroutineCall;

impl OpCodeReader for SubroutineCall {
    fn opcode_val(&self) -> u16 {
        0x2000
    }

    fn opcode_mask(&self) -> u16 {
        0xf000
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        state.stack.push_back(state.pc);
        state.pc = Address(opcode_data.nnn);
    }
}

#[derive(Debug, Default, Clone)]
pub struct SubroutineReturn;

impl OpCodeReader for SubroutineReturn {
    fn opcode_val(&self) -> u16 {
        0x00EE
    }

    fn opcode_mask(&self) -> u16 {
        0xFFFF
    }

    fn execute(&self, state: &mut Chip8State, _opcode_data: OpCodeData) {
        let return_address = state.stack.pop_back().expect("no elements to pop");
        state.pc = return_address;
    }
}

#[derive(Debug, Default, Clone)]
pub struct SkipConstEqual;

impl OpCodeReader for SkipConstEqual {
    fn opcode_val(&self) -> u16 {
        0x3000
    }

    fn opcode_mask(&self) -> u16 {
        0xF000
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        if state.gp_register(opcode_data.x).0 == opcode_data.nn {
            state.pc += 2;
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SkipConstNotEqual;

impl OpCodeReader for SkipConstNotEqual {
    fn opcode_val(&self) -> u16 {
        0x4000
    }

    fn opcode_mask(&self) -> u16 {
        0xF000
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        if state.gp_register(opcode_data.x).0 != opcode_data.nn {
            state.pc += 2;
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SkipRegistersEqual;

impl OpCodeReader for SkipRegistersEqual {
    fn opcode_val(&self) -> u16 {
        0x5000
    }

    fn opcode_mask(&self) -> u16 {
        0xF00F
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        if state.gp_register(opcode_data.x).0 == state.gp_register(opcode_data.y).0 {
            state.pc += 2;
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SkipRegistersNotEqual;

impl OpCodeReader for SkipRegistersNotEqual {
    fn opcode_val(&self) -> u16 {
        0x9000
    }

    fn opcode_mask(&self) -> u16 {
        0xF00F
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        if state.gp_register(opcode_data.x).0 != state.gp_register(opcode_data.y).0 {
            state.pc += 2;
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SetRegisterRegister;

impl OpCodeReader for SetRegisterRegister {
    fn opcode_val(&self) -> u16 {
        0x8000
    }

    fn opcode_mask(&self) -> u16 {
        0xF00F
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        state.gp_register(opcode_data.x).0 = state.gp_register(opcode_data.y).0;
    }
}

#[derive(Debug, Default, Clone)]
pub struct BinaryOr;

impl OpCodeReader for BinaryOr {
    fn opcode_val(&self) -> u16 {
        0x8001
    }

    fn opcode_mask(&self) -> u16 {
        0xF00F
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        state.gp_register(opcode_data.x).0 |= state.gp_register(opcode_data.y).0;
    }
}

#[derive(Debug, Default, Clone)]
pub struct BinaryAnd;

impl OpCodeReader for BinaryAnd {
    fn opcode_val(&self) -> u16 {
        0x8002
    }

    fn opcode_mask(&self) -> u16 {
        0xF00F
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        state.gp_register(opcode_data.x).0 &= state.gp_register(opcode_data.y).0;
    }
}

#[derive(Debug, Default, Clone)]
pub struct BinaryXor;

impl OpCodeReader for BinaryXor {
    fn opcode_val(&self) -> u16 {
        0x8003
    }

    fn opcode_mask(&self) -> u16 {
        0xF00F
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        state.gp_register(opcode_data.x).0 ^= state.gp_register(opcode_data.y).0;
    }
}

#[derive(Debug, Default, Clone)]
pub struct AddRegisters;

impl OpCodeReader for AddRegisters {
    fn opcode_val(&self) -> u16 {
        0x8004
    }

    fn opcode_mask(&self) -> u16 {
        0xF00F
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        let x_reg_val = state.gp_register(opcode_data.x).0;
        let y_reg_val = state.gp_register(opcode_data.y).0;
        let sat_add = x_reg_val.saturating_add(y_reg_val);
        let wrap_add = x_reg_val.wrapping_add(y_reg_val);
        state.gp_register(opcode_data.x).0 = wrap_add;
        state.gp_register(0xF).0 = if sat_add != wrap_add { 0x1 } else { 0x0 };
    }
}

#[derive(Debug, Default, Clone)]
pub struct SubtractRegisters;

impl OpCodeReader for SubtractRegisters {
    fn opcode_val(&self) -> u16 {
        0x8005
    }

    fn opcode_mask(&self) -> u16 {
        0xF00F
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        let x_reg_val = state.gp_register(opcode_data.x).0;
        let y_reg_val = state.gp_register(opcode_data.y).0;
        state.gp_register(opcode_data.x).0 = x_reg_val.wrapping_sub(y_reg_val);
        state.gp_register(0xF).0 = if y_reg_val > x_reg_val { 0x0 } else { 0x1 };
    }
}

#[derive(Debug, Default, Clone)]
pub struct SubtractRegistersReverse;

impl OpCodeReader for SubtractRegistersReverse {
    fn opcode_val(&self) -> u16 {
        0x8007
    }

    fn opcode_mask(&self) -> u16 {
        0xF00F
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        let x_reg_val = state.gp_register(opcode_data.x).0;
        let y_reg_val = state.gp_register(opcode_data.y).0;
        state.gp_register(opcode_data.x).0 = y_reg_val.wrapping_sub(x_reg_val);
        state.gp_register(0xF).0 = if x_reg_val > y_reg_val { 0x0 } else { 0x1 };
    }
}

#[derive(Debug, Default, Clone)]
pub struct ShiftRegisterRight;

impl OpCodeReader for ShiftRegisterRight {
    fn opcode_val(&self) -> u16 {
        0x8006
    }

    fn opcode_mask(&self) -> u16 {
        0xF00F
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        let x_reg = state.gp_register(opcode_data.x);
        let removed_bit = x_reg.0 & 0x01;
        x_reg.0 >>= 1;
        state.gp_register(0xF).0 = removed_bit;
    }
}

#[derive(Debug, Default, Clone)]
pub struct ShiftRegisterLeft;

impl OpCodeReader for ShiftRegisterLeft {
    fn opcode_val(&self) -> u16 {
        0x800E
    }

    fn opcode_mask(&self) -> u16 {
        0xF00F
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        let x_reg = state.gp_register(opcode_data.x);
        let removed_bit = if (x_reg.0 & 0x80) == 0 { 0x00 } else { 0x01 };
        x_reg.0 <<= 1;
        state.gp_register(0xF).0 = removed_bit;
    }
}

#[derive(Debug, Default, Clone)]
pub struct JumpOffset;

impl OpCodeReader for JumpOffset {
    fn opcode_val(&self) -> u16 {
        0xB000
    }

    fn opcode_mask(&self) -> u16 {
        0xF000
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        state.pc = Address(state.pc.0 + opcode_data.nnn + u16::from(state.gp_register(0x0).0));
    }
}

#[derive(Debug, Default, Clone)]
pub struct Random;

impl OpCodeReader for Random {
    fn opcode_val(&self) -> u16 {
        0xC000
    }

    fn opcode_mask(&self) -> u16 {
        0xF000
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        state.gp_register(opcode_data.x).0 = rand::random::<u8>() & opcode_data.nn;
    }
}

#[derive(Debug, Default, Clone)]
pub struct SkipIfKey;

impl OpCodeReader for SkipIfKey {
    fn opcode_val(&self) -> u16 {
        0xE09E
    }

    fn opcode_mask(&self) -> u16 {
        0xF0FF
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        let key = state.gp_register(opcode_data.x).0;
        if state.is_pressed(key) {
            state.pc.0 += 2;
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SkipIfNotKey;

impl OpCodeReader for SkipIfNotKey {
    fn opcode_val(&self) -> u16 {
        0xE0A1
    }

    fn opcode_mask(&self) -> u16 {
        0xF0FF
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        let key = state.gp_register(opcode_data.x).0;
        if !state.is_pressed(key) {
            log::debug!("SkipIfNotKey: skipping (key {key:#x})");
            state.pc.0 += 2;
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ReadDelayTimer;

impl OpCodeReader for ReadDelayTimer {
    fn opcode_val(&self) -> u16 {
        0xF007
    }

    fn opcode_mask(&self) -> u16 {
        0xF0FF
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        state.gp_register(opcode_data.x).0 = state.delay_timer.0;
    }
}

#[derive(Debug, Default, Clone)]
pub struct SetDelayTimer;

impl OpCodeReader for SetDelayTimer {
    fn opcode_val(&self) -> u16 {
        0xF015
    }

    fn opcode_mask(&self) -> u16 {
        0xF0FF
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        state.delay_timer.0 = state.gp_register(opcode_data.x).0;
    }
}

#[derive(Debug, Default, Clone)]
pub struct SetSoundTimer;

impl OpCodeReader for SetSoundTimer {
    fn opcode_val(&self) -> u16 {
        0xF018
    }

    fn opcode_mask(&self) -> u16 {
        0xF0FF
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        state.sound_timer.0 = state.gp_register(opcode_data.x).0;
    }
}

#[derive(Debug, Default, Clone)]
pub struct AddIndexRegister;

impl OpCodeReader for AddIndexRegister {
    fn opcode_val(&self) -> u16 {
        0xF01E
    }

    fn opcode_mask(&self) -> u16 {
        0xF0FF
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        state.index_register.0 += u16::from(state.gp_register(opcode_data.x).0);
        let overflows = state.index_register.0 > 0xFFF;
        state.gp_register(0xF).0 = if overflows { 0x1 } else { 0x0 }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GetKey;

impl OpCodeReader for GetKey {
    fn opcode_val(&self) -> u16 {
        0xF01A
    }

    fn opcode_mask(&self) -> u16 {
        0xF0FF
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        let key = state.gp_register(opcode_data.x).0;
        if !state.is_pressed(key) {
            state.pc.0 -= 2;
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ReadFontCharacter;

impl OpCodeReader for ReadFontCharacter {
    fn opcode_val(&self) -> u16 {
        0xF029
    }

    fn opcode_mask(&self) -> u16 {
        0xF0FF
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        state.index_register.0 = 0x50 + (u16::from(state.gp_register(opcode_data.x).0) * 0x5);
    }
}

#[derive(Debug, Default, Clone)]
pub struct DecimalDecoding;

impl OpCodeReader for DecimalDecoding {
    fn opcode_val(&self) -> u16 {
        0xF033
    }

    fn opcode_mask(&self) -> u16 {
        0xF0FF
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        let register_val = state.gp_register(opcode_data.x).0;
        let digits = [
            register_val / 100,
            (register_val % 100) / 10,
            register_val % 10,
        ];
        state.memory_set(&digits, state.index_register);
    }
}

#[derive(Debug, Default, Clone)]
pub struct StoreMemory;

impl OpCodeReader for StoreMemory {
    fn opcode_val(&self) -> u16 {
        0xF055
    }

    fn opcode_mask(&self) -> u16 {
        0xF0FF
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        let address_start = usize::from(state.index_register.0);
        for reg in 0..=opcode_data.x {
            state.memory[address_start + usize::from(reg)] = state.gp_register(reg).0;
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct LoadMemory;

impl OpCodeReader for LoadMemory {
    fn opcode_val(&self) -> u16 {
        0xF065
    }

    fn opcode_mask(&self) -> u16 {
        0xF0FF
    }

    fn execute(&self, state: &mut Chip8State, opcode_data: OpCodeData) {
        let address_start = usize::from(state.index_register.0);
        for reg in 0..=opcode_data.x {
            state.gp_register(reg).0 = state.memory[address_start + usize::from(reg)];
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        display::{Coordinates, Display},
        emulator::{Address, Register},
    };
    use expect_test::expect;
    use std::collections::VecDeque;
    use test_case::test_case;

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
            expected_screen.assert_eq(&state.display.to_string());
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
    fn test_set_register_const() {
        let sr_reader = SetRegisterConst;
        let mut state = Chip8State::new().with_register(Register(0xef), 2);
        let correct_state = state.clone().with_register(Register(0x12), 2);
        sr_reader.execute(&mut state, OpCodeData::decode(0x6212));
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_add_register_const() {
        let ar_reader = AddRegisterConst;
        let mut state = Chip8State::new().with_register(Register(0x43), 0x0a);
        let correct_state = state.clone().with_register(Register(0x7d), 0x0a);
        ar_reader.execute(&mut state, OpCodeData::decode(0x7a3a));
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_set_index_register() {
        let sir_reader = SetIndexRegister;
        let mut state = Chip8State::new().with_index_register(Address(0x001));
        let correct_state = state.clone().with_index_register(Address(0x0123));
        sir_reader.execute(&mut state, OpCodeData::decode(0xA123));
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_subroutine_call() {
        let subroutine_call_reader = SubroutineCall;
        let mut state = Chip8State::new().with_pc(Address(0x100));
        let correct_state = state
            .clone()
            .with_pc(Address(0x123))
            .with_stack([Address(0x100)].into_iter().collect());
        subroutine_call_reader.execute(&mut state, OpCodeData::decode(0x2123));
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_subroutine_return() {
        let subroutine_return_reader = SubroutineReturn;
        let mut state = Chip8State::new()
            .with_pc(Address(0x123))
            .with_stack([Address(0x100)].into_iter().collect());
        let correct_state = state
            .clone()
            .with_pc(Address(0x100))
            .with_stack(VecDeque::default());
        subroutine_return_reader.execute(&mut state, OpCodeData::decode(0x00EE));
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_skip_const_equal() {
        let skip_const_equal_reader = SkipConstEqual;
        let original_state = Chip8State::new()
            .with_pc(Address(0x100))
            .with_register(Register(0x5A), 0x3);
        let skip_state = original_state.clone().with_pc(Address(0x102));

        // Case with skip
        {
            let mut state = original_state.clone();
            skip_const_equal_reader.execute(&mut state, OpCodeData::decode(0x335A));
            assert_eq!(state, skip_state);
        }

        // Case without skip
        {
            let mut state = original_state.clone();
            skip_const_equal_reader.execute(&mut state, OpCodeData::decode(0x334A));
            assert_eq!(state, original_state);
        }
    }

    #[test]
    fn test_skip_const_not_equal() {
        let skip_const_not_equal_reader = SkipConstNotEqual;
        let original_state = Chip8State::new()
            .with_pc(Address(0x100))
            .with_register(Register(0x5A), 0x3);
        let skip_state = original_state.clone().with_pc(Address(0x102));

        // Case with skip
        {
            let mut state = original_state.clone();
            skip_const_not_equal_reader.execute(&mut state, OpCodeData::decode(0x434A));
            assert_eq!(state, skip_state);
        }

        // Case without skip
        {
            let mut state = original_state.clone();
            skip_const_not_equal_reader.execute(&mut state, OpCodeData::decode(0x435A));
            assert_eq!(state, original_state);
        }
    }

    #[test]
    fn test_skip_registers_equal() {
        let skip_registers_equal_reader = SkipRegistersEqual;
        let original_state = Chip8State::new()
            .with_pc(Address(0x100))
            .with_register(Register(0x5A), 0x3)
            .with_register(Register(0x5A), 0x4)
            .with_register(Register(0x4B), 0x5);
        let skip_state = original_state.clone().with_pc(Address(0x102));

        // Case with skip
        {
            let mut state = original_state.clone();
            skip_registers_equal_reader.execute(&mut state, OpCodeData::decode(0x5340));
            assert_eq!(state, skip_state);
        }

        // Case without skip
        {
            let mut state = original_state.clone();
            skip_registers_equal_reader.execute(&mut state, OpCodeData::decode(0x5350));
            assert_eq!(state, original_state);
        }
    }

    #[test]
    fn test_skip_registers_not_equal() {
        let skip_registers_not_equal_reader = SkipRegistersNotEqual;
        let original_state = Chip8State::new()
            .with_pc(Address(0x100))
            .with_register(Register(0x5A), 0x3)
            .with_register(Register(0x5A), 0x4)
            .with_register(Register(0x4B), 0x5);
        let skip_state = original_state.clone().with_pc(Address(0x102));

        // Case with skip
        {
            let mut state = original_state.clone();
            skip_registers_not_equal_reader.execute(&mut state, OpCodeData::decode(0x9350));
            assert_eq!(state, skip_state);
        }

        // Case without skip
        {
            let mut state = original_state.clone();
            skip_registers_not_equal_reader.execute(&mut state, OpCodeData::decode(0x9340));
            assert_eq!(state, original_state);
        }
    }

    #[test]
    fn test_set_register_register() {
        let set_register_register_reader = SetRegisterRegister;
        let mut state = Chip8State::new()
            .with_pc(Address(0x100))
            .with_register(Register(0x9C), 0x2)
            .with_register(Register(0xC6), 0x3);
        let correct_state = state.clone().with_register(Register(0xC6), 0x02);
        set_register_register_reader.execute(&mut state, OpCodeData::decode(0x8230));
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_binary_or() {
        let binary_or_reader = BinaryOr;
        let mut state = Chip8State::new()
            .with_pc(Address(0x100))
            .with_register(Register(0x9C), 0x2)
            .with_register(Register(0xC6), 0x3);
        let correct_state = state.clone().with_register(Register(0xDE), 0x02);
        binary_or_reader.execute(&mut state, OpCodeData::decode(0x8231));
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_binary_and() {
        let binary_and_reader = BinaryAnd;
        let mut state = Chip8State::new()
            .with_pc(Address(0x100))
            .with_register(Register(0x9C), 0x2)
            .with_register(Register(0xC6), 0x3);
        let correct_state = state.clone().with_register(Register(0x84), 0x02);
        binary_and_reader.execute(&mut state, OpCodeData::decode(0x8232));
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_binary_xor() {
        let binary_xor_reader = BinaryXor;
        let mut state = Chip8State::new()
            .with_pc(Address(0x100))
            .with_register(Register(0x9C), 0x2)
            .with_register(Register(0xC6), 0x3);
        let correct_state = state.clone().with_register(Register(0x5A), 0x02);
        binary_xor_reader.execute(&mut state, OpCodeData::decode(0x8233));
        assert_eq!(state, correct_state);
    }

    #[test_case(0x4B, 0x17, 0x62, false, 0x00; "normal")]
    #[test_case(0xA2, 0x7F, 0x21, true,  0x00; "overflow")]
    #[test_case(0x4B, 0x17, 0x62, false, 0xDF; "normal_carry_override")]
    #[test_case(0xA2, 0x7F, 0x21, true,  0xDF; "overflow_carry_override")]
    fn test_add_registers(a: u8, b: u8, result: u8, overflows: bool, vf_value: u8) {
        let add_registers_reader = AddRegisters;
        let mut state = Chip8State::new()
            .with_pc(Address(0x100))
            .with_register(Register(a), 0x2)
            .with_register(Register(b), 0x3)
            .with_register(Register(vf_value), 0xf);
        let correct_state = state
            .clone()
            .with_register(Register(result), 0x2)
            .with_register(Register(if overflows { 0x01 } else { 0x00 }), 0xF);
        add_registers_reader.execute(&mut state, OpCodeData::decode(0x8234));
        assert_eq!(state, correct_state);
    }

    // Note here that result should be a = a - b (aka X = X - Y)
    #[test_case(0xC3, 0x4D, 0x76, true,  0x00; "normal")]
    #[test_case(0x11, 0xCA, 0x47, false, 0x00; "underflow")]
    #[test_case(0xC3, 0x4D, 0x76, true,  0xDF; "normal_carry_override")]
    #[test_case(0x11, 0xCA, 0x47, false, 0xDF; "underflow_carry_override")]
    fn test_subtract_registers(a: u8, b: u8, result: u8, underflows: bool, vf_value: u8) {
        let subtract_registers_reader = SubtractRegisters;
        let mut state = Chip8State::new()
            .with_pc(Address(0x100))
            .with_register(Register(a), 0x2)
            .with_register(Register(b), 0x3)
            .with_register(Register(vf_value), 0xf);
        let correct_state = state
            .clone()
            .with_register(Register(result), 0x2)
            .with_register(Register(if underflows { 0x01 } else { 0x00 }), 0xF);
        subtract_registers_reader.execute(&mut state, OpCodeData::decode(0x8235));
        assert_eq!(state, correct_state);
    }

    // Note here that result should be a = b - a (aka X = Y - X)
    #[test_case(0x4D, 0xC3, 0x76, true,  0x00; "normal")]
    #[test_case(0xCA, 0x11, 0x47, false, 0x00; "underflow")]
    #[test_case(0x4D, 0xC3, 0x76, true,  0xDF; "normal_carry_override")]
    #[test_case(0xCA, 0x11, 0x47, false, 0xDF; "underflow_carry_override")]
    fn test_subtract_registers_reverse(a: u8, b: u8, result: u8, underflows: bool, vf_value: u8) {
        let subtract_registers_reverse_reader = SubtractRegistersReverse;
        let mut state = Chip8State::new()
            .with_pc(Address(0x100))
            .with_register(Register(a), 0x2)
            .with_register(Register(b), 0x3)
            .with_register(Register(vf_value), 0xf);
        let correct_state = state
            .clone()
            .with_register(Register(result), 0x2)
            .with_register(Register(if underflows { 0x01 } else { 0x00 }), 0xF);
        subtract_registers_reverse_reader.execute(&mut state, OpCodeData::decode(0x8237));
        assert_eq!(state, correct_state);
    }

    #[test_case(0x9C,  0x4E, false, 0x00;  "normal")]
    #[test_case(0x59,  0x2C, true,  0x00;  "bit_shifted")]
    #[test_case(0x9C,  0x4E, false, 0xDF;  "normal_shift_override")]
    #[test_case(0x59,  0x2C, true,  0xDF;  "bit_shifted_shift_override")]
    fn test_shift_register_right(val: u8, result: u8, bit_shifted: bool, vf_value: u8) {
        let shift_register_right_reader = ShiftRegisterRight;
        let mut state = Chip8State::new()
            .with_register(Register(val), 0x7)
            .with_register(Register(vf_value), 0xf);
        let correct_state = state
            .clone()
            .with_register(Register(result), 0x7)
            .with_register(Register(if bit_shifted { 0x01 } else { 0x00 }), 0xF);
        shift_register_right_reader.execute(&mut state, OpCodeData::decode(0x8706));
        assert_eq!(state, correct_state);
    }

    #[test_case(0x59, 0xB2, false, 0x00;  "normal")]
    #[test_case(0x9C, 0x38, true,  0x00;  "bit_shifted")]
    #[test_case(0x59, 0xB2, false, 0xDF;  "normal_shift_override")]
    #[test_case(0x9C, 0x38, true,  0xDF;  "bit_shifted_shift_override")]
    fn test_shift_register_left(val: u8, result: u8, bit_shifted: bool, vf_value: u8) {
        let shift_register_left_reader = ShiftRegisterLeft;
        let mut state = Chip8State::new()
            .with_register(Register(val), 0x7)
            .with_register(Register(vf_value), 0xf);
        let correct_state = state
            .clone()
            .with_register(Register(result), 0x7)
            .with_register(Register(if bit_shifted { 0x01 } else { 0x00 }), 0xF);
        shift_register_left_reader.execute(&mut state, OpCodeData::decode(0x870E));
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_jump_offset() {
        let jump_offset_reader = JumpOffset;
        let mut state = Chip8State::new()
            .with_pc(Address(0x100))
            .with_register(Register(0x12), 0x0);
        let correct_state = state.clone().with_pc(Address(0x266));
        jump_offset_reader.execute(&mut state, OpCodeData::decode(0xB154));
        assert_eq!(state, correct_state);
    }

    #[test_case(0xA, 0x1, 0x100; "wrong_key_pressed")]
    #[test_case(0xF, 0xF, 0x102; "key_pressed")]
    fn test_skip_if_key(key_pressed: u8, key_checked: u8, expected_pc: u16) {
        let skip_if_key_reader = SkipIfKey;
        let mut state = Chip8State::new()
            .with_pc(Address(0x100))
            .with_key_pressed(key_pressed)
            .with_register(Register(key_checked), 0x5);
        let correct_state = state.clone().with_pc(Address(expected_pc));
        skip_if_key_reader.execute(&mut state, OpCodeData::decode(0xE59E));
        assert_eq!(state, correct_state);
    }

    #[test_case(0xA, 0x1, 0x102; "key_not_pressed")]
    #[test_case(0xF, 0xF, 0x100; "key_pressed")]
    fn test_skip_if_not_key(key_pressed: u8, key_checked: u8, expected_pc: u16) {
        let skip_if_not_key_reader = SkipIfNotKey;
        let mut state = Chip8State::new()
            .with_pc(Address(0x100))
            .with_key_pressed(key_pressed)
            .with_register(Register(key_checked), 0x5);
        let correct_state = state.clone().with_pc(Address(expected_pc));
        skip_if_not_key_reader.execute(&mut state, OpCodeData::decode(0xE59E));
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_read_delay_timer() {
        let read_delay_timer_reader = ReadDelayTimer;
        let mut state = Chip8State::new().with_delay_timer(Register(0x9F));
        let correct_state = state.clone().with_register(Register(0x9F), 0x5);
        read_delay_timer_reader.execute(&mut state, OpCodeData::decode(0xF507));
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_set_delay_timer() {
        let set_delay_timer_reader = SetDelayTimer;
        let mut state = Chip8State::new().with_register(Register(0x9F), 0x4);
        let correct_state = state.clone().with_delay_timer(Register(0x9F));
        set_delay_timer_reader.execute(&mut state, OpCodeData::decode(0xF415));
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_set_sound_timer() {
        let set_sound_timer_reader = SetSoundTimer;
        let mut state = Chip8State::new().with_register(Register(0x9F), 0x2);
        let correct_state = state.clone().with_sound_timer(Register(0x9F));
        set_sound_timer_reader.execute(&mut state, OpCodeData::decode(0xF218));
        assert_eq!(state, correct_state);
    }

    #[test_case(0x11F, 0xA5, 0x1C4,  false; "normal")]
    #[test_case(0xFAF, 0xA5, 0x1054, true;  "overflow")]
    fn test_add_index_register(idx_val: u16, register_val: u8, result: u16, overflows: bool) {
        let add_index_register_reader = AddIndexRegister;
        let mut state = Chip8State::new()
            .with_register(Register(register_val), 0xA)
            .with_index_register(Address(idx_val));
        let correct_state = state
            .clone()
            .with_index_register(Address(result))
            .with_register(Register(if overflows { 0x1 } else { 0x0 }), 0xF);
        add_index_register_reader.execute(&mut state, OpCodeData::decode(0xFA1E));
        assert_eq!(state, correct_state);
    }

    #[test_case(0xD, 0x2, 0xFE; "key_not_pressed")]
    #[test_case(0x5, 0x5, 0x100; "key_pressed")]
    fn test_get_key(pressed_key: u8, checked_key: u8, expected_pc: u16) {
        let get_key_reader = GetKey;
        let mut state = Chip8State::new()
            .with_pc(Address(0x100))
            .with_key_pressed(pressed_key)
            .with_register(Register(checked_key), 0xD);
        let correct_state = state.clone().with_pc(Address(expected_pc));
        get_key_reader.execute(&mut state, OpCodeData::decode(0xFD0A));
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_read_font_character() {
        let read_font_character_reader = ReadFontCharacter;
        let mut state = Chip8State::new().with_register(Register(0x7), 0xB);
        // We expect our font to be loaded starting at address 0x50, and each "character" is 5
        // bytes long, so 0x50 + (0x7 * 0x5) = 0x73
        let correct_state = state.clone().with_index_register(Address(0x073));
        read_font_character_reader.execute(&mut state, OpCodeData::decode(0xFB29));
        assert_eq!(state, correct_state);
    }

    #[test_case(255, &[2, 5, 5], 0x123; "three_digits")]
    #[test_case(13, &[0, 1, 3], 0x200; "two_digits")]
    #[test_case(9, &[0, 0, 9], 0xDFF; "one_digit")]
    fn test_decimal_decoding(value: u8, digits: &'static [u8; 3], address: u16) {
        let decimal_decoding_reader = DecimalDecoding;
        let mut state = Chip8State::new()
            .with_register(Register(value), 0x8)
            .with_index_register(Address(address));
        let correct_state = state.clone().with_memory_set(digits, Address(address));
        decimal_decoding_reader.execute(&mut state, OpCodeData::decode(0xF833));
        assert_eq!(state, correct_state);
    }

    const SAMPLE_DATA: &[u8] = &[
        0xDE, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE,
        0xFF,
    ];

    #[test_case(0x123, 0x5; "six_bytes")]
    #[test_case(0x500, 0xF; "all_bytes")]
    #[test_case(0xFFF, 0x0; "one_byte")]
    fn test_store_memory(address: u16, register: u8) {
        let store_memory_reader = StoreMemory;
        let mut state = Chip8State::new()
            .with_index_register(Address(address))
            .with_register(Register(0xDE), 0x0)
            .with_register(Register(0x11), 0x1)
            .with_register(Register(0x22), 0x2)
            .with_register(Register(0x33), 0x3)
            .with_register(Register(0x44), 0x4)
            .with_register(Register(0x55), 0x5)
            .with_register(Register(0x66), 0x6)
            .with_register(Register(0x77), 0x7)
            .with_register(Register(0x88), 0x8)
            .with_register(Register(0x99), 0x9)
            .with_register(Register(0xAA), 0xA)
            .with_register(Register(0xBB), 0xB)
            .with_register(Register(0xCC), 0xC)
            .with_register(Register(0xDD), 0xD)
            .with_register(Register(0xEE), 0xE)
            .with_register(Register(0xFF), 0xF);
        let correct_state = state
            .clone()
            .with_memory_set(&SAMPLE_DATA[..=usize::from(register)], Address(address));
        store_memory_reader.execute(
            &mut state,
            OpCodeData::decode(0xF055 + u16::from(register) * 0x100),
        );
        assert_eq!(state, correct_state);
    }

    #[test_case(0x123, 0x5; "six_bytes")]
    #[test_case(0xFFF, 0x0; "one_byte")]
    #[test_case(0x500, 0xF; "all_bytes")]
    fn test_load_memory(address: u16, register: u8) {
        let load_memory_reader = LoadMemory;
        let mut state = Chip8State::new()
            .with_memory_set(&SAMPLE_DATA[..=usize::from(register)], Address(address))
            .with_index_register(Address(address));
        let mut correct_state = state.clone();
        for reg in 0..=register {
            correct_state =
                correct_state.with_register(Register(SAMPLE_DATA[usize::from(reg)]), reg);
        }
        load_memory_reader.execute(
            &mut state,
            OpCodeData::decode(0xF065 + u16::from(register) * 0x100),
        );
        assert_eq!(state, correct_state);
    }

    #[test]
    fn test_display_draw_basic() {
        let mut state = get_draw_state();
        let d_reader = DisplayDraw;
        // We'll draw from 56,8, resulting in: (from 52,8):
        //|    ████ ███|
        //|    █  █  ██|
        //|████  █   ██|
        //|██████████  |
        //|██████████  |
        //[etc.]
        // Basic case, confirm it gets drawn
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
            |                                                    ████  █   ██|
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

    #[test]
    fn test_display_draw_coordinate_wraps() {
        let d_reader = DisplayDraw;
        // This time we'll draw from x=248 (248 = 56 + 2*64), and y = 136 (8 + 3*32). Should draw
        // the exact same diagram
        let mut state = get_draw_state()
            .with_register(Register(120), 2) // x
            .with_register(Register(136), 3); // y

        // Basic case, confirm it gets drawn
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
            |                                                    ████  █   ██|
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

    #[test]
    fn test_display_draw_sprite_truncates() {
        let d_reader = DisplayDraw;
        let mut state = get_draw_state()
            .with_register(Register(57), 2) // x coordinate
            .with_register(Register(8), 3); // y coordinate
                                            // ██ █████  (0b11011111 = 0xDF)

        // We'll draw from 57,8. This causes our sprite to trunkate, resulting in: (from 52,8):
        //|     ████ ██|
        //|     █  █  █|
        //|█████  █  ██|
        //|██████████  |
        //|██████████  |
        //[etc.]
        // truncation case
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
            |                                                         ████ ██|
            |                                                         █  █  █|
            |                                                    █████  █  ██|
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

    fn get_draw_state() -> Chip8State {
        let display = {
            let mut display = Display::default();
            display.flip_all(Coordinates::new(52, 10), Coordinates::new(61, 19));
            display
        };
        let mut state = Chip8State::new()
            .with_display(display)
            .with_index_register(Address(0x300))
            .with_register(Register(56), 2) // Stores X
            .with_register(Register(8), 3); // Stores Y

        // Currently we have a box of 10x10 starting at 56,8
        // The sprite we store looks like the following (remember all sprites are 8 bytes in width):
        // ████ ███  (0b11110111 = 0xF7)
        // █  █  ██  (0b10010011 = 0x93)
        // ██ █████  (0b11011111 = 0xDF)
        state.memory[0x300..0x303].copy_from_slice(&[0xF7, 0x93, 0xDF]);
        // "random" data to ensure we don't read past end
        state.memory[0x303..0x306].copy_from_slice(&[0xAB, 0x41, 0x9A]);

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

        state
    }
}

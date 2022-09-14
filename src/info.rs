/**
5       1   4       4        4        14
xxxxx   x   xxxx    xxxx     xxxx     xxxxxxxxxxxxxx
opcode I=0 dst_reg src1_reg  src2_reg dont_care_bits

5       1   4       4        2       16
xxxxx   x   xxxx    xxxx     xx      xxxxxxxxxxxxxxxx
opcode I=1 dst_reg src1_reg  modbits immediate(src2)

5      27
xxxx   xxxxxxxxxxxxxxxxxxxxxxxxxxx
opcode word_offset(2's complement)
*/

// Offsets of fields
pub const OPCODE_OFF: u8 = 27;
pub const IMMBIT_OFF: u8 = 26;
pub const DST_OFF: u8 = 22;
pub const SRC1_OFF: u8 = 18;
pub const MOD_OFF: u8 = 16;
pub const SRC2_OFF: u8 = 14;

// Field width in bits
pub const OPCODE_BITS: u8 = 5;
pub const IMMBIT_BITS: u8 = 1;
pub const REG_BITS: u8 = 4;
pub const MOD_BITS: u8 = 2;
pub const IMM_BITS: u8 = 16;
pub const OFFSET_BITS: u8 = 27;

pub const MOD_DEF: u8 = 0b00;
pub const MOD_U: u8 = 0b01;
pub const MOD_H: u8 = 0b10;
pub const RET_REG_ID: usize = 15;
pub const WORD_BITS: u8 = 32;

pub mod opcodes {
    pub const ADD: u8 = 0;
    pub const SUB: u8 = 1;
    pub const MUL: u8 = 2;
    pub const DIV: u8 = 3;
    pub const MOD: u8 = 4;
    pub const CMP: u8 = 5;
    pub const AND: u8 = 6;
    pub const OR: u8 = 7;
    pub const NOT: u8 = 8;
    pub const MOV: u8 = 9;
    pub const LSL: u8 = 10;
    pub const LSR: u8 = 11;
    pub const ASR: u8 = 12;
    pub const NOP: u8 = 13;
    pub const LD: u8 = 14;
    pub const ST: u8 = 15;
    pub const BEQ: u8 = 16;
    pub const BGT: u8 = 17;
    pub const B: u8 = 18;
    pub const CALL: u8 = 19;
    pub const RET: u8 = 20;
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Instruction {
    pub name: &'static str,
    pub opcode: u8,
    pub ndst: u8,
    pub nsrc: u8,
    pub modbits: u8,
}

macro_rules! instup {
    ($name:literal, $opcode:literal, $ndst:literal, $nsrc:literal) => {
        Instruction {
            name: $name,
            opcode: $opcode,
            ndst: $ndst,
            nsrc: $nsrc,
            modbits: 0u8,
        }
    };
}

pub const INSTRUCTIONS: [Instruction; 21] = [
    // Support u an h modifiers
    instup!("add", 0, 1, 2),
    instup!("sub", 1, 1, 2),
    instup!("mul", 2, 1, 2),
    instup!("div", 3, 1, 2),
    instup!("mod", 4, 1, 2),
    instup!("cmp", 5, 0, 2),
    instup!("and", 6, 1, 2),
    instup!("or", 7, 1, 2),
    instup!("not", 8, 1, 1),
    instup!("mov", 9, 1, 1),
    instup!("lsl", 10, 1, 2),
    instup!("lsr", 11, 1, 2),
    instup!("asr", 12, 1, 2),
    // These below do not support u and h modifiers
    instup!("nop", 13, 0, 0),
    instup!("ld", 14, 1, 2),
    instup!("st", 15, 1, 2),
    // src(if allowed) is a label
    instup!("beq", 16, 0, 1),
    instup!("bgt", 17, 0, 1),
    instup!("b", 18, 0, 1),
    instup!("call", 19, 0, 1),
    instup!("ret", 20, 0, 0),
];

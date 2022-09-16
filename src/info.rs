//! Contains information about all instructions.

/*!
Instruction Encoding:
Each instruction is 32 bits long and there are 3 total formats.
Not all instructions use every field.

```text
5       1   4       4        4        14
xxxxx   x   xxxx    xxxx     xxxx     xxxxxxxxxxxxxx
opcode I=0 dst_reg src1_reg  src2_reg dont_care_bits

5       1   4       4        2       16
xxxxx   x   xxxx    xxxx     xx      xxxxxxxxxxxxxxxx
opcode I=1 dst_reg src1_reg  modbits immediate(src2)

5      27
xxxx   xxxxxxxxxxxxxxxxxxxxxxxxxxx
opcode word_offset(2's complement)
```
*/

/**
The system-call instruction
sys is a 0-address instruction.

It uses the following registers for argument passing and return values:
Sycall number : r0
Argument[1-4] : r[1-4]
Return value  : r0

Information about system calls is documented in the simpleRISC.md file
 */

pub const RET_REG: usize = 15;

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

// Immediate modifier bits values
pub const MOD_DEF: u8 = 0b00;
pub const MOD_U: u8 = 0b01;
pub const MOD_H: u8 = 0b10;

pub mod opcodes {
    // Must be in order
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
    pub const SYS: u8 = 21;
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
    ($name:literal, $opcode:expr, $ndst:literal, $nsrc:literal) => {
        Instruction {
            name: $name,
            opcode: $opcode,
            ndst: $ndst,
            nsrc: $nsrc,
            modbits: 0u8,
        }
    };
}

use opcodes::*;
// Must be in the same order as in opcodes
pub const INSTRUCTIONS: [Instruction; 22] = [
    // These Instructions(upto mov) support 'u' & 'h' modifiers
    instup!("add", ADD, 1, 2),
    instup!("sub", SUB, 1, 2),
    instup!("mul", MUL, 1, 2),
    instup!("div", DIV, 1, 2),
    instup!("mod", MOD, 1, 2),
    instup!("cmp", CMP, 0, 2),
    instup!("and", AND, 1, 2),
    instup!("or", OR, 1, 2),
    instup!("not", NOT, 1, 1),
    instup!("mov", MOV, 1, 1),
    // Instructions below do not support the modifiers
    instup!("lsl", LSL, 1, 2),
    instup!("lsr", LSR, 1, 2),
    instup!("asr", ASR, 1, 2),
    instup!("nop", NOP, 0, 0),
    instup!("ld", LD, 1, 2),
    instup!("st", ST, 1, 2),
    // src for branch instructions is a label
    instup!("beq", BEQ, 0, 1),
    instup!("bgt", BGT, 0, 1),
    instup!("b", B, 0, 1),
    instup!("call", CALL, 0, 1),
    instup!("ret", RET, 0, 0),
    instup!("sys", SYS, 0, 0),
];

pub fn support_mod(opcode: u8) -> bool {
    opcode <= MOV
}

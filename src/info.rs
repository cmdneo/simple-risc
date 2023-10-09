//! Contains information about all instructions.

/*!
Instruction Encoding:
Each instruction is 32 bits long and there are 3 different formats.

3-Address, src2 is a register:
```text
5       1   4       4        4        14
bbbbb   0   bbbb    bbbb     bbbb     xxxxxxxxxxxxxx
opcode I=0 dst_reg src1_reg  src2_reg dont_care
```

3-Address, src2 is an immediate:
```text
5       1   4       4        2       16
bbbbb   1   bbbb    bbbb     bb      bbbbbbbbbbbbbbbb
opcode I=1 dst_reg src1_reg  modbits immediate(src2)
```

2-address:
Same as 3-address except that src1_reg field is not used(dont_care)

1-Address:
```text
5      27
bbbb   bbbbbbbbbbbbbbbbbbbbbbbbbbb
opcode word_offset(2's complement)
```

0-address
```text
5      27
bbbb   xxxxxxxxxxxxxxxxxxxxxxxxxxx
opcode dont_care
```
*/

/**
Extra:
The system-call instruction.
sys is a 0-address instruction.

It uses the following registers for argument passing and return values:
Sycall number : r0
Argument[1-4] : r[1-4]
Return value  : r0

Information about system calls is documented in the simpleRISC.md file
 */

pub const RET_REG: usize = 15;

pub mod bits {
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
    // Immediate modifier bits(2-bits)
    pub const MOD_DEF: u8 = 0b00;
    pub const MOD_U: u8 = 0b01;
    pub const MOD_H: u8 = 0b10;
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, PartialOrd, Clone, Copy)]
pub enum Opcode {
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    CMP,
    AND,
    OR,
    NOT,
    MOV,
    LSL,
    LSR,
    ASR,
    NOP,
    LD,
    ST,
    BEQ,
    BGT,
    B,
    CALL,
    RET,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Instruction {
    pub name: &'static str,
    pub opcode: Opcode,
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

use Opcode::*;
// Must be in the same order as in opcodes
pub const INSTRUCTIONS: [Instruction; 21] = [
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
];

pub fn supports_mod(opcode: u8) -> bool {
    opcode <= MOV as u8
}

pub fn supports_imm(opcode: u8) -> bool {
    let ins = INSTRUCTIONS[opcode as usize];
    ins.ndst + ins.nsrc >= 2
}

//! Implements a basic (maybe working) emulator for simpleRISC.
//! It uses 2's complement wrap-around arithmetic for all calculations.

use crate::info::{self, opcodes::*};
use std::{
    io::{Read, Write},
    num::Wrapping,
};

const MEM_WORD_MAX: usize = 4096;

struct UnpackedIns {
    dst_reg: usize,
    src1: Wrapping<i32>,
    src2: Wrapping<i32>,
    memaddr: i32,
    new_pc: i32,
    opcode: u8,
}

pub struct Emulator<'a> {
    /// Register file, r[0-15]
    regs: [Wrapping<i32>; 16],
    /// Stores words(=4bytes) instead of storing each byte seperately.
    /// Only for aligned(by 4 bytes) access, word_index = memaddr/4
    wmemory: [Wrapping<i32>; MEM_WORD_MAX],
    instructions: &'a [u32],
    prog_cnt: i32,
    flag_e: bool,
    flag_g: bool,
}

#[derive(Debug)]
pub enum EmulatorErr {
    InvalidModbits,
    InvalidMemAddr,
    InvalidOpcode,
    InvalidSyscall,
    DivideByZero,
    UnalignedMemAddr,
}

fn get_bits(bits: u32, n: u8, offset: u8) -> u32 {
    (bits >> offset) & (!0u32 >> (32 - n))
}

fn sign_extend(num: u32, nbits: u8) -> i32 {
    if num >> (nbits - 1) != 0 {
        (num | (!0u32 << nbits)) as i32
    } else {
        num as i32
    }
}

impl<'a> Emulator<'a> {
    pub fn from(instructions: &'a [u32]) -> Self {
        Self {
            regs: [Wrapping(0); 16],
            wmemory: [Wrapping(0); 4096],
            instructions,
            prog_cnt: 0,
            flag_e: false,
            flag_g: false,
        }
    }

    pub fn debug(&self) {
        for (i, &rval) in self.regs.iter().enumerate() {
            println!("r{:<2} = {}", i, rval);
        }
    }

    pub fn get_reg_val(&self, reg_num: usize) -> i32 {
        self.regs[reg_num].0
    }

    pub fn exec(&mut self) -> Result<(), EmulatorErr> {
        while self.prog_cnt >= 0 && (self.prog_cnt as usize) < self.instructions.len() {
            self.prog_cnt = self.exec_ins(self.instructions[self.prog_cnt as usize])?;
        }
        Ok(())
    }

    fn exec_ins(&mut self, bits: u32) -> Result<i32, EmulatorErr> {
        let UnpackedIns {
            dst_reg,
            src1,
            mut src2,
            memaddr,
            new_pc,
            mut opcode,
        } = self.decode_fetch(bits)?;

        // Modify and verify fields as needed
        match opcode {
            // Convert BGT and BEQ to NOP if corresponding flags not set
            BEQ if !self.flag_e => opcode = NOP,
            BGT if !self.flag_g => opcode = NOP,
            // Only consider the lower 5 bits for shift amount(that is max 31)
            LSL | LSR | ASR => src2 = Wrapping(src2.0 & 0b11111),
            DIV | MOD if src2.0 == 0 => return Err(EmulatorErr::DivideByZero),
            _ => {}
        };

        self.regs[dst_reg] = match opcode {
            ADD => src1 + src2,
            SUB => src1 - src2,
            MUL => src1 * src2,
            DIV => src1 / src2,
            MOD => src1 % src2,
            CMP => {
                self.flag_e = src1 == src2;
                self.flag_g = src1 > src2;
                self.regs[dst_reg]
            }
            AND => src1 & src2,
            OR => src1 | src2,
            NOT => !src2,
            MOV => src2,
            LSL => Wrapping(src1.0 << src2.0),
            LSR => Wrapping(((src1.0 as u32) >> src2.0) as i32),
            ASR => Wrapping(src1.0 >> src2.0),
            NOP => self.regs[dst_reg],
            LD => self.wmemory[self.get_word_index(memaddr)?],
            ST => {
                self.wmemory[self.get_word_index(memaddr)?] = self.regs[dst_reg];
                self.regs[dst_reg]
            }
            // Conditional branch instructions are already converted to NOPs if flags not set
            BEQ | BGT | B => return Ok(new_pc),
            CALL => {
                self.regs[info::RET_REG] = Wrapping(self.prog_cnt + 1);
                return Ok(new_pc);
            }
            RET => return Ok(self.regs[info::RET_REG].0),
            SYS => Wrapping(self.do_syscall(src2.0)?),
            _ => {
                return Err(EmulatorErr::InvalidOpcode);
            }
        };
        Ok(self.prog_cnt + 1)
    }

    fn get_word_index(&self, memaddr: i32) -> Result<usize, EmulatorErr> {
        if memaddr < 0 {
            return Err(EmulatorErr::InvalidMemAddr);
        }
        if memaddr % 4 != 0 {
            return Err(EmulatorErr::UnalignedMemAddr);
        }
        // A word is 4 bytes
        let word_idx = (memaddr as usize) / 4;
        if word_idx >= MEM_WORD_MAX {
            return Err(EmulatorErr::InvalidMemAddr);
        }
        Ok(word_idx)
    }

    fn decode_fetch(&self, bits: u32) -> Result<UnpackedIns, EmulatorErr> {
        // See src/info.rs for more info
        let opcode = get_bits(bits, info::OPCODE_BITS, info::OPCODE_OFF) as u8;
        // Branch instructions cannot have immediates(their encoding is different),
        // so is_imm is always false for them
        let is_imm = match opcode {
            B | BEQ | BGT | CALL | RET => false,
            _ => get_bits(bits, info::IMMBIT_BITS, info::IMMBIT_OFF) == 1,
        };
        let modbits = get_bits(bits, info::MOD_BITS, info::MOD_OFF) as u8;
        let dst_reg = get_bits(bits, info::REG_BITS, info::DST_OFF) as usize;
        let src1 = self.regs[get_bits(bits, info::REG_BITS, info::SRC1_OFF) as usize];
        // src2 can be either a register or an immediate
        let tmps2 = if is_imm {
            let imm = get_bits(bits, info::IMM_BITS, 0);
            match modbits {
                info::MOD_DEF => sign_extend(imm, info::IMM_BITS),
                info::MOD_U => imm as i32,
                info::MOD_H => (imm << u16::BITS) as i32,
                _ => return Err(EmulatorErr::InvalidModbits),
            }
        } else {
            self.regs[get_bits(bits, info::REG_BITS, info::SRC2_OFF) as usize].0
        };
        let src2 = Wrapping(tmps2);
        let new_pc =
            self.prog_cnt + sign_extend(get_bits(bits, info::OFFSET_BITS, 0), info::OFFSET_BITS);
        // imm[reg] is understood as (reg + imm)
        let memaddr = src1 + src2;

        Ok(UnpackedIns {
            dst_reg,
            src1,
            src2,
            memaddr: memaddr.0,
            new_pc,
            opcode,
        })
    }

    fn do_syscall(&mut self, call_num: i32) -> Result<i32, EmulatorErr> {
        let ret = match call_num {
            0 => sys_getchar(),
            1 => sys_putchar(self.regs[1].0),
            _ => return Err(EmulatorErr::InvalidSyscall),
        };
        Ok(ret)
    }
}

// All system call functions take i32 type for all arguments
fn sys_getchar() -> i32 {
    if let Some(Ok(b)) = std::io::stdin().bytes().next() {
        b as i32
    } else {
        -1
    }
}

fn sys_putchar(c: i32) -> i32 {
    if let Ok(_) = std::io::stdout().write(&[c as u8]) {
        c
    } else {
        -1
    }
}

#[cfg(test)]
mod tests {
    use super::sign_extend;
    #[test]
    fn test_sign_extent() {
        assert_eq!(sign_extend(0b11111, 5), -1);
        assert_eq!(sign_extend(0b10000, 5), -16);
        assert_eq!(sign_extend(0b01111, 5), 15);
    }
}

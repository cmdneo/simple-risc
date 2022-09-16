use crate::info::{self, opcodes, Instruction};
use std::{collections::HashMap, num::IntErrorKind};

const REGISTERS: [(&str, u8); 17] = [
    ("r0", 0),
    ("r1", 1),
    ("r2", 2),
    ("r3", 3),
    ("r4", 4),
    ("r5", 5),
    ("r6", 6),
    ("r7", 7),
    ("r8", 8),
    ("r9", 9),
    ("r10", 10),
    ("r11", 11),
    ("r12", 12),
    ("r13", 13),
    ("r14", 14),
    ("sp", 14), // Alias for r14
    ("r15", 15),
];

#[derive(Debug, PartialEq)]
enum Token {
    Eof,
    Ident(String),
    Inst(Instruction),
    Reg(u8),
    Imm(u16),
    Char(char),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseErr {
    NoMatch,
    NoMainFound,
    IllegalModifier,
    ImmediateOverflow,
    InvalidImmediate,
    UnexpectedToken,
    DuplicateLabel(String),
    UndefinedLabel(String),
    RegisterExpected,
    ImmediateExpected,
    OperandExpected,
    IdentifierExpected,
    CharExpected(char),
}

impl Token {
    fn try_imm(self) -> Result<u16, ParseErr> {
        if let Self::Imm(imm) = self {
            Ok(imm)
        } else {
            Err(ParseErr::ImmediateExpected)
        }
    }

    fn try_reg(self) -> Result<u8, ParseErr> {
        if let Self::Reg(reg) = self {
            Ok(reg)
        } else {
            Err(ParseErr::RegisterExpected)
        }
    }

    fn try_ident(self) -> Result<String, ParseErr> {
        if let Self::Ident(ident) = self {
            Ok(ident)
        } else {
            Err(ParseErr::IdentifierExpected)
        }
    }

    fn try_the_char(self, mc: char) -> Result<char, ParseErr> {
        if let Self::Char(c) = self {
            if c == mc {
                return Ok(mc);
            }
        }
        Err(ParseErr::CharExpected(mc))
    }
}

#[derive(Debug, PartialEq)]
enum Operand {
    Label(String),
    Imm(u16),
    Reg(u8),
}

#[derive(Debug, PartialEq)]
struct Statement {
    inst: Instruction,
    dst: u8,
    src1: u8,
    src2: Operand,
}

struct Scanner {
    chars: Vec<char>,
    cursor: usize,
    line: usize,
}

impl Scanner {
    fn from(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            cursor: 0,
            line: 1,
        }
    }

    fn peek(&self) -> Option<&char> {
        self.chars.get(self.cursor)
    }

    fn peekn(&self, n: usize) -> Option<&char> {
        self.chars.get(self.cursor + n)
    }

    fn next(&mut self) -> Option<&char> {
        if let Some('\n') = self.peek() {
            self.line += 1
        }
        self.cursor += 1;
        self.chars.get(self.cursor - 1)
    }
}

pub struct Parser {
    citer: Scanner,
    stmt_cnt: usize,
    labels: HashMap<String, usize>,
}

impl Parser {
    pub fn from(code: &str) -> Self {
        Self {
            citer: Scanner::from(code),
            labels: HashMap::new(),
            stmt_cnt: 0,
        }
    }

    pub fn print_err(&mut self, err: ParseErr) {
        match err {
            // Not detected at parse time, so no info about line & col
            ParseErr::UndefinedLabel(_) => {
                eprintln!("Error: {:?}", err)
            }
            _ => eprintln!("Error on line {}: {:?}", self.citer.line, err),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<u32>, ParseErr> {
        let mut stmts: Vec<Statement> = Vec::new();

        loop {
            match self.next_tok()? {
                Token::Ident(ident) => {
                    self.next_tok()?.try_the_char(':')?;
                    if self.labels.contains_key(&ident) {
                        return Err(ParseErr::DuplicateLabel(ident));
                    }
                    self.labels.insert(ident, self.stmt_cnt);
                }
                Token::Inst(inst) => {
                    stmts.push(self.statement(inst)?);
                }
                Token::Char('\n') => { /* Ignore extra newlines */ }
                Token::Eof => break,
                _ => return Err(ParseErr::UnexpectedToken),
            };
        }
        self.assemble(stmts)
    }

    fn assemble(&self, stmts: Vec<Statement>) -> Result<Vec<u32>, ParseErr> {
        let mut ret: Vec<u32> = Vec::new();

        for Statement {
            inst,
            dst,
            src1,
            src2,
        } in stmts
        {
            let tmp = match (inst.ndst, inst.nsrc) {
                (1, 2) => encode_rrx(inst.opcode, dst, src1, inst.modbits, src2),
                (1, 1) => encode_rrx(inst.opcode, dst, 0, inst.modbits, src2),
                (0, 2) => encode_rrx(inst.opcode, 0, src1, inst.modbits, src2),
                (0, 1) => encode_offset(inst.opcode, self.get_label_pos(src2)?, ret.len()),
                (0, 0) => encode_rrx(inst.opcode, 0, 0, 0, Operand::Reg(0)),
                (_, _) => panic!("Parser.parse encoder: If you are seeing this then RUN!"),
            };
            ret.push(tmp);
        }
        Ok(ret)
    }

    fn get_label_pos(&self, label_op: Operand) -> Result<usize, ParseErr> {
        if let Operand::Label(ident) = label_op {
            if let Some(&label_at) = self.labels.get(&ident) {
                Ok(label_at)
            } else {
                Err(ParseErr::UndefinedLabel(ident))
            }
        } else {
            panic!("Non-label operand passed to get_label_pos");
        }
    }

    fn next_tok(&mut self) -> Result<Token, ParseErr> {
        while let Some(&c) = self.citer.peek() {
            if c == '\t' || c == ' ' || c == '@' {
                // If a comment then skip to the end of the line
                if c == '@' {
                    collect_chars_while(&mut self.citer, |&c| c != '\n');
                } else {
                    self.citer.next();
                }
                continue;
            }

            return match c {
                '+' | '-' | '0'..='9' => immediate(&mut self.citer),
                c if is_ident_char(&c) => identifier(&mut self.citer),
                c => {
                    self.citer.next();
                    Ok(Token::Char(c))
                }
            };
        }
        Ok(Token::Eof)
    }

    fn statement(&mut self, inst: Instruction) -> Result<Statement, ParseErr> {
        let (mut dst, mut src1, mut src2) = (0u8, 0u8, Operand::Reg(0));
        let is_ldst = matches!(inst.opcode, opcodes::LD | opcodes::ST);
        // Label only instructions take only one source and no destination
        let is_op2_label = inst.ndst == 0 && inst.nsrc == 1;

        // := reg
        //  | reg ',' # If at least one source operand
        if inst.ndst == 1 {
            dst = self.next_tok()?.try_reg()?;
            if inst.nsrc > 0 {
                self.next_tok()?.try_the_char(',')?;
            }
        }
        // := imm '[' reg ']'
        if is_ldst {
            src2 = Operand::Imm(self.next_tok()?.try_imm()?);
            self.next_tok()?.try_the_char('[')?;
            src1 = self.next_tok()?.try_reg()?;
            self.next_tok()?.try_the_char(']')?;
        }
        // := ident
        else if is_op2_label {
            src2 = Operand::Label(self.next_tok()?.try_ident()?);
        }
        // := reg
        //  | imm
        else if inst.nsrc == 1 {
            src2 = match self.next_tok()? {
                Token::Reg(reg) => Operand::Reg(reg),
                Token::Imm(imm) => Operand::Imm(imm),
                _ => return Err(ParseErr::OperandExpected),
            };
        }
        // := reg ',' reg
        //  | reg ',' imm
        else if inst.nsrc == 2 {
            src1 = self.next_tok()?.try_reg()?;
            self.next_tok()?.try_the_char(',')?;
            src2 = match self.next_tok()? {
                Token::Reg(reg) => Operand::Reg(reg),
                Token::Imm(imm) => Operand::Imm(imm),
                _ => return Err(ParseErr::OperandExpected),
            };
        }
        // If no immediate operand and modifier is present, then illegal
        if let Operand::Imm(_) = src2 {
            // Immediate operand, fine
        } else {
            if inst.modbits != info::MOD_DEF {
                return Err(ParseErr::IllegalModifier);
            }
        }
        // Each statement is terminated by a newline
        self.next_tok()?.try_the_char('\n')?;
        self.stmt_cnt += 1;

        Ok(Statement {
            inst,
            dst,
            src1,
            src2,
        })
    }
}

fn encode_rrx(opcode: u8, dst: u8, src1: u8, modbits: u8, src2: Operand) -> u32 {
    match src2 {
        Operand::Reg(regs2) => {
            (opcode as u32) << info::OPCODE_OFF
                | (dst as u32) << info::DST_OFF
                | (src1 as u32) << info::SRC1_OFF
                | (regs2 as u32) << info::SRC2_OFF
        }
        Operand::Imm(imm) => {
            (opcode as u32) << info::OPCODE_OFF
                | 1 << info::IMMBIT_OFF
                | (dst as u32) << info::DST_OFF
                | (src1 as u32) << info::SRC1_OFF
                | (modbits as u32) << info::MOD_OFF
                | (imm as u32)
        }
        Operand::Label(_) => panic!("This function cannot encode Operand::Label types"),
    }
}

fn encode_offset(opcode: u8, label_at: usize, cur_at: usize) -> u32 {
    let offset = (label_at as i32 - cur_at as i32) as u32;
    (opcode as u32) << info::OPCODE_OFF | (offset & (!0u32 >> info::OPCODE_BITS))
}

/// take_while consumes an extra character at last so..., use this
fn collect_chars_while(citer: &mut Scanner, pred: fn(ch: &char) -> bool) -> String {
    let mut ret = String::new();
    while let Some(&ch) = citer.peek() {
        if pred(&ch) {
            ret.push(ch);
            citer.next();
        } else {
            break;
        }
    }
    ret
}

fn immediate(citer: &mut Scanner) -> Result<Token, ParseErr> {
    let mut base = 10;
    let mut is_neg = false;
    let num: u16;

    match citer.peek() {
        Some(&c) if c == '+' || c == '-' => {
            citer.next();
            is_neg = c == '-';
        }
        _ => {}
    }
    if let Some('0') = citer.peek() {
        match citer.peekn(1) {
            Some('x') => base = 16,
            Some('o') => base = 8,
            Some('b') => base = 2,
            _ => {}
        }
        if base != 10 {
            citer.next();
            citer.next();
        }
    }

    let num_str = collect_chars_while(citer, char::is_ascii_alphanumeric);
    match u16::from_str_radix(num_str.as_str(), base) {
        Ok(ntmp) => {
            num = ntmp;
        }
        Err(e) => match e.kind() {
            IntErrorKind::NegOverflow | IntErrorKind::PosOverflow => {
                return Err(ParseErr::ImmediateOverflow)
            }
            // Also handles if string is empty
            _ => return Err(ParseErr::InvalidImmediate),
        },
    }
    if is_neg {
        // Check for overflow and then convert to 2's Complement representation
        if num > std::i16::MIN.unsigned_abs() {
            return Err(ParseErr::ImmediateOverflow);
        }
        return Ok(Token::Imm(!num + 1));
    }
    Ok(Token::Imm(num))
}

fn is_ident_char(c: &char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '$')
}

fn identifier(citer: &mut Scanner) -> Result<Token, ParseErr> {
    let ident = collect_chars_while(citer, is_ident_char);
    // Can be a register name or instruction name
    if let Some(&reg) = REGISTERS.iter().find(|&&reg| reg.0 == ident) {
        return Ok(Token::Reg(reg.1));
    }
    match instruction(&ident) {
        Ok(tok) => return Ok(tok),
        Err(ParseErr::NoMatch) => { /* Nothing matched, move on */ }
        Err(e) => return Err(e),
    }
    Ok(Token::Ident(ident))
}

fn instruction(mut instr: &str) -> Result<Token, ParseErr> {
    let modbits: u8;

    if instr.ends_with('u') {
        instr = instr.strip_suffix('u').unwrap();
        modbits = info::MOD_U;
    } else if instr.ends_with('h') {
        instr = instr.strip_suffix('h').unwrap();
        modbits = info::MOD_H;
    } else {
        modbits = info::MOD_DEF;
    }

    for Instruction {
        name,
        opcode,
        ndst,
        nsrc,
        modbits: _,
    } in info::INSTRUCTIONS
    {
        if instr != name {
            continue;
        }
        // Modifiers are supported only by ALU(except shift) and mov instructions
        if modbits != info::MOD_DEF && !info::support_mod(opcode) {
            return Err(ParseErr::IllegalModifier);
        }

        return Ok(Token::Inst(Instruction {
            name,
            opcode,
            ndst,
            nsrc,
            modbits,
        }));
    }

    Err(ParseErr::NoMatch)
}

#[cfg(test)]
mod tests {
    use super::immediate;
    use super::instruction;
    use super::Instruction;
    use super::ParseErr;
    use super::Scanner;
    use super::Token;

    #[test]
    fn imm_test() {
        let test_pairs: [(&str, Result<Token, ParseErr>); 6] = [
            ("42", Ok(Token::Imm(42))),
            ("0", Ok(Token::Imm(0))),
            ("-42", Ok(Token::Imm(!42 + 1))),
            ("-0x69", Ok(Token::Imm(!0x69 + 1))),
            ("0x1FFFF", Err(ParseErr::ImmediateOverflow)),
            ("0x1oops", Err(ParseErr::InvalidImmediate)),
        ];
        for (test, res) in test_pairs {
            assert_eq!(immediate(&mut Scanner::from(test)), res);
        }
    }

    #[test]
    fn instruction_test() {
        assert_eq!(
            instruction("add"),
            Ok(Token::Inst(Instruction {
                name: "add",
                opcode: 0,
                ndst: 1,
                nsrc: 2,
                modbits: super::info::MOD_DEF,
            }))
        );
        assert_eq!(
            instruction("addh"),
            Ok(Token::Inst(Instruction {
                name: "add",
                opcode: 0,
                ndst: 1,
                nsrc: 2,
                modbits: super::info::MOD_H,
            }))
        );
        // Invalid ones
        assert_eq!(instruction("nopu"), Err(ParseErr::IllegalModifier));
        assert_eq!(instruction("nosuchins"), Err(ParseErr::NoMatch));
    }
}

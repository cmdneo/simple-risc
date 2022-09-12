use std::{collections::HashMap, num::IntErrorKind};

pub const WORD_BITS: u8 = 32;
const OPCODE_OFF: u8 = 27;
const IMM_BIT_OFF: u8 = 26;
const DST_OFF: u8 = 22;
const SRC1_OFF: u8 = 18;
const MOD_OFF: u8 = 16;
const SRC2_OFF: u8 = 14;

const MOD_DEF: u8 = 0b00;
const MOD_U: u8 = 0b01;
const MOD_H: u8 = 0b10;

const REGISTERS: [(&str, u8); 16] = [
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
    ("r15", 15),
];

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

#[derive(PartialEq, Debug)]
struct Instruction {
    name: &'static str,
    opcode: u8,
    ndst: u8,
    nsrc: u8,
    modbits: u8,
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

const INSTRUCTIONS: [Instruction; 21] = [
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

#[derive(Debug, PartialEq)]
enum Token {
    Eof,
    Ident(String),
    Inst(Instruction),
    Reg(u8),
    Imm(u16),
    Char(char),
}

#[derive(Debug, PartialEq)]
pub enum ParseErr {
    NoMatch,
    ImmOverflow,
    InvalidImm,
    InvalidMod,
    ImmediateExpected,
    RegisterExpected,
    IdentifierExpected,
    DuplicateLabel,
    UndefinedLabel,
    OperandExpected,
    UnexpectedToken,
    CharExpected(char),
}

// Write a macro or some better solution
impl Token {
    fn try_imm(&self) -> Result<u16, ParseErr> {
        if let Token::Imm(imm) = self {
            return Ok(*imm);
        }
        return Err(ParseErr::ImmediateExpected);
    }

    fn try_reg(&self) -> Result<u8, ParseErr> {
        if let Token::Reg(reg) = self {
            return Ok(*reg);
        }
        return Err(ParseErr::RegisterExpected);
    }

    fn try_ident(&self) -> Result<String, ParseErr> {
        if let Token::Ident(ident) = self {
            return Ok(ident.clone());
        }
        return Err(ParseErr::IdentifierExpected);
    }

    fn try_the_char(&self, mc: char) -> Result<char, ParseErr> {
        if let Token::Char(c) = self {
            if *c == mc {
                return Ok(mc);
            }
        }
        return Err(ParseErr::CharExpected(mc));
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
}

impl Scanner {
    fn from(input: &str) -> Self {
        Scanner {
            chars: input.chars().collect(),
            cursor: 0,
        }
    }

    fn peek(&self) -> Option<&char> {
        if self.cursor != self.chars.len() {
            Some(&self.chars[self.cursor])
        } else {
            None
        }
    }

    fn peekn(&self, n: usize) -> Option<&char> {
        if self.cursor + n < self.chars.len() {
            Some(&self.chars[self.cursor + n])
        } else {
            None
        }
    }

    fn next(&mut self) -> Option<&char> {
        self.cursor += 1;
        self.peek()
    }
}

pub struct Parser {
    citer: Scanner,
    stmt_cnt: usize,
    cursor: usize,
    line: usize,
    col: usize,
    labels: HashMap<String, usize>,
}

impl Parser {
    pub fn from(input: &str) -> Self {
        Parser {
            citer: Scanner::from(input),
            labels: HashMap::new(),
            stmt_cnt: 0,
            cursor: 0,
            line: 1,
            col: 0,
        }
    }

    pub fn print_err(&mut self, err: ParseErr) {
        eprintln!("Error on line {}:{}: {:?}", self.line, self.col, err);
    }

    pub fn parse(&mut self) -> Result<Vec<u32>, ParseErr> {
        let mut ret: Vec<u32> = Vec::new();
        let mut stmts: Vec<Statement> = Vec::new();

        loop {
            match self.next_tok()? {
                Token::Ident(ident) => {
                    self.next_tok()?.try_the_char(':')?;
                    if let Some(_) = self.labels.insert(ident, self.stmt_cnt) {
                        return Err(ParseErr::DuplicateLabel);
                    }
                    continue;
                }
                Token::Inst(inst) => {
                    stmts.push(self.statement(inst)?);
                }
                Token::Char('\n') => { /* Ignore extra newlines */ }
                Token::Eof => break,
                _ => return Err(ParseErr::UnexpectedToken),
            };
        }

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
                (_, _) => panic!("If you are seeing this then RUN!"),
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
                Err(ParseErr::UndefinedLabel)
            }
        } else {
            panic!("Non-label operand passed to get_label_pos");
        }
    }

    fn next_tok(&mut self) -> Result<Token, ParseErr> {
        while let Some(&c) = self.citer.peek() {
            self.col += 1;
            self.cursor += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 0;
            }
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
                c if c.is_ascii_alphabetic() => identifier(&mut self.citer),
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
        let is_ldst = [opcodes::LD, opcodes::ST].contains(&inst.opcode);
        let is_op2_label = inst.ndst == 0 && inst.nsrc == 1;

        // := reg
        //  | reg ',' # If at least one source operand
        if inst.ndst == 1 {
            dst = self.next_tok()?.try_reg()?;
            if inst.nsrc > 0 {
                self.next_tok()?.try_the_char(',')?;
            }
        }
        // := imm[reg]
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
        //  | reg ',' reg
        //  | reg ',' imm
        else if inst.nsrc > 0 {
            src1 = self.next_tok()?.try_reg()?;
            if inst.nsrc == 2 {
                self.next_tok()?.try_the_char(',')?;
                let tmp = self.next_tok()?;
                // reg or imm
                if let Token::Reg(reg) = tmp {
                    src2 = Operand::Reg(reg);
                } else if let Token::Imm(imm) = tmp {
                    src2 = Operand::Imm(imm)
                } else {
                    return Err(ParseErr::OperandExpected);
                }
            }
        }
        self.next_tok()?.try_the_char('\n')?;
        self.stmt_cnt += 1;
        Ok(Statement {
            inst,
            dst,
            src1,
            src2,
        })
    }

    // fn next_token(mut self) -> Token<'a> {}
}

fn encode_rrx(opcode: u8, dst: u8, src1: u8, modbits: u8, src2: Operand) -> u32 {
    match src2 {
        Operand::Reg(regs2) => {
            (opcode as u32) << OPCODE_OFF
                | (dst as u32) << DST_OFF
                | (src1 as u32) << SRC1_OFF
                | (regs2 as u32) << SRC2_OFF
        }
        Operand::Imm(imm) => {
            (opcode as u32) << OPCODE_OFF
                | 1 << IMM_BIT_OFF
                | (dst as u32) << DST_OFF
                | (src1 as u32) << SRC1_OFF
                | (modbits as u32) << MOD_OFF
                | (imm as u32)
        }
        Operand::Label(_) => panic!("This function cannot do Operand::Label"),
    }
}

fn encode_offset(opcode: u8, label_at: usize, cur_at: usize) -> u32 {
    let offset = (label_at as i32 - cur_at as i32) as u32;
    (opcode as u32) << OPCODE_OFF | (offset & (!0u32 >> OPCODE_OFF))
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
                return Err(ParseErr::ImmOverflow)
            }
            // Also handles if string is empty
            _ => return Err(ParseErr::InvalidImm),
        },
    }
    if is_neg {
        // Check for overflow and then convert to 2's Complement representation
        if num > (std::i16::MIN as i32).abs() as u16 {
            return Err(ParseErr::ImmOverflow);
        }
        return Ok(Token::Imm(!num + 1));
    }
    Ok(Token::Imm(num))
}

fn identifier(citer: &mut Scanner) -> Result<Token, ParseErr> {
    let ident = collect_chars_while(citer, |&c| c.is_ascii_alphanumeric() || c == '_');
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

fn instruction(mut iname: &str) -> Result<Token, ParseErr> {
    let mut modbits: u8 = MOD_DEF;

    if iname.ends_with('u') {
        iname = iname.strip_suffix('u').unwrap();
        modbits = MOD_U;
    } else if iname.ends_with('h') {
        iname = iname.strip_suffix('h').unwrap();
        modbits = MOD_H;
    }

    for Instruction {
        name,
        opcode,
        ndst,
        nsrc,
        modbits: _,
    } in INSTRUCTIONS
    {
        if iname != name {
            continue;
        }
        // Immediate modifiers are supported only for instructions with
        // opcode 12 or less(before NOP)
        if modbits != MOD_DEF && opcode >= opcodes::NOP {
            return Err(ParseErr::InvalidMod);
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
    use crate::parser::Scanner;

    use super::immediate;
    use super::instruction;
    use super::Instruction;
    use super::ParseErr;
    use super::Token;

    #[test]
    fn imm_test() {
        let test_pairs: [(&str, Result<Token, ParseErr>); 6] = [
            ("42", Ok(Token::Imm(42))),
            ("0", Ok(Token::Imm(0))),
            ("-42", Ok(Token::Imm(!42 + 1))),
            ("-0x69", Ok(Token::Imm(!0x69 + 1))),
            ("0x1FFFF", Err(ParseErr::ImmOverflow)),
            ("0x1oops", Err(ParseErr::InvalidImm)),
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
                modbits: super::MOD_DEF,
            }))
        );
        assert_eq!(
            instruction("addh"),
            Ok(Token::Inst(Instruction {
                name: "add",
                opcode: 0,
                ndst: 1,
                nsrc: 2,
                modbits: super::MOD_H,
            }))
        );
        // Invalid ones
        assert_eq!(instruction("nopu"), Err(ParseErr::InvalidMod));
        assert_eq!(instruction("nosuchins"), Err(ParseErr::NoMatch));
    }
}

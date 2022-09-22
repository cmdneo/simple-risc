use crate::info::{self, bits, opcodes, Instruction};
use std::{collections::HashMap, fmt, num::IntErrorKind};

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

#[derive(Debug, PartialEq, Eq)]
pub struct ParseErr {
    kind: ErrKind,
    line: usize,
}

#[derive(Debug, PartialEq, Eq)]
enum ErrKind {
    IllegalModifier,
    ImmOverflow,
    InvalidImm,
    OpenComment,
    RegExp,
    ImmExp,
    OperandExp,
    IdentExp,
    IllegalToken,
    CharExp(char),
    DuplicateLabel(String),
    UndefinedLabel(String),
}

impl std::error::Error for ParseErr {}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrKind::UndefinedLabel(_) => write!(f, ""),
            _ => write!(f, "On line {}: ", self.line),
        }?;

        match &self.kind {
            ErrKind::IllegalModifier => write!(f, "Modifier not allowed"),
            ErrKind::ImmOverflow => write!(f, "Immediate out of range(overflow)"),
            ErrKind::InvalidImm => write!(f, "Invalid immediate"),
            ErrKind::OpenComment => write!(f, "Comment not closed"),
            ErrKind::IllegalToken => write!(f, "Token not expected by any rule"),
            ErrKind::RegExp => write!(f, "Register Expected"),
            ErrKind::ImmExp => write!(f, "Immediate Expected"),
            ErrKind::OperandExp => write!(f, "Immediate or register expected"),
            ErrKind::IdentExp => write!(f, "Label expected"),
            ErrKind::CharExp(c) => write!(f, "Character '{}' expected", c),
            ErrKind::DuplicateLabel(s) => write!(f, "Duplicate label '{}'", s),
            ErrKind::UndefinedLabel(s) => write!(f, "Label not found '{}'", s),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Token {
    Eof,
    Ident(String),
    Inst(Instruction),
    Reg(u8),
    Imm(u16),
    Char(char),
}

impl Token {
    fn try_imm(self) -> Result<u16, ErrKind> {
        if let Self::Imm(imm) = self {
            Ok(imm)
        } else {
            Err(ErrKind::ImmExp)
        }
    }

    fn try_reg(self) -> Result<u8, ErrKind> {
        if let Self::Reg(reg) = self {
            Ok(reg)
        } else {
            Err(ErrKind::RegExp)
        }
    }

    fn try_ident(self) -> Result<String, ErrKind> {
        if let Self::Ident(ident) = self {
            Ok(ident)
        } else {
            Err(ErrKind::IdentExp)
        }
    }

    fn try_the_char(self, mc: char) -> Result<char, ErrKind> {
        if let Self::Char(c) = self {
            if c == mc {
                return Ok(mc);
            }
        }
        Err(ErrKind::CharExp(mc))
    }
}

#[derive(Debug, PartialEq)]
enum Operand {
    Label(String),
    Imm(u16),
    Reg(u8),
}

struct Statement {
    inst: Instruction,
    dst: u8,
    src1: u8,
    src2: Operand,
}

struct Scanner<'a> {
    left: &'a str,
    cursor: usize,
    line: usize,
    col: usize,
}

impl<'a> Scanner<'a> {
    fn new(input: &'a str) -> Self {
        Scanner {
            left: input,
            cursor: 0,
            line: 1,
            col: 1,
        }
    }

    /// Consume the input until `pred` evaluates to true and return the consumed part
    fn take_while(&mut self, pred: impl Fn(char) -> bool) -> &'a str {
        let mut end = 0usize;

        for (i, ch) in self.left.char_indices() {
            if !pred(ch) {
                break;
            }
            end = i + ch.len_utf8();
            self.update_cursor(ch)
        }
        let ret;
        (ret, self.left) = self.left.split_at(end);
        ret
    }

    /// If the input starts with `prefix`, then consume it and
    /// return true, otherwise return false
    fn eat_prefix(&mut self, prefix: &str) -> bool {
        if let Some(suf) = self.left.strip_prefix(prefix) {
            prefix.chars().for_each(|ch| self.update_cursor(ch));
            self.left = suf;
            true
        } else {
            false
        }
    }

    /// Returns the first char without consuming input
    fn peek(&self) -> Option<char> {
        self.peekn(0)
    }

    /// Returns the nth char without consuming input
    fn peekn(&self, n: usize) -> Option<char> {
        self.left.chars().nth(n)
    }

    /// Consume the character and return it
    fn next(&mut self) -> Option<char> {
        let mut iter = self.left.chars();
        let ret = iter.next();
        self.left = iter.as_str();
        ret
    }

    fn update_cursor(&mut self, ch: char) {
        self.cursor += 1;
        self.col += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 0;
        }
    }
}

struct Parser<'a> {
    scn: Scanner<'a>,
    labels: HashMap<String, usize>,
    stmt_cnt: usize,
}

impl<'a> Parser<'a> {
    pub fn new(code: &'a str) -> Self {
        Self {
            scn: Scanner::new(code),
            labels: HashMap::new(),
            stmt_cnt: 0,
        }
    }

    pub fn line_num(&self) -> usize {
        self.scn.line
    }

    pub fn parse(&mut self) -> Result<Vec<u32>, ErrKind> {
        let mut stmts: Vec<Statement> = Vec::new();

        loop {
            match self.next_tok()? {
                Token::Ident(ident) => {
                    self.next_tok()?.try_the_char(':')?;
                    if self.labels.contains_key(&ident) {
                        return Err(ErrKind::DuplicateLabel(ident));
                    }
                    self.labels.insert(ident, self.stmt_cnt);
                }
                Token::Inst(inst) => stmts.push(self.make_statement(inst)?),
                Token::Char('\n') => { /* Ignore extra newlines */ }
                Token::Eof => break,
                _ => return Err(ErrKind::IllegalToken),
            };
        }
        self.assemble(stmts)
    }

    fn assemble(&self, stmts: Vec<Statement>) -> Result<Vec<u32>, ErrKind> {
        let mut ret: Vec<u32> = Vec::new();

        for Statement {
            inst,
            dst,
            src1,
            src2,
        } in stmts
        {
            let tmp = match (inst.ndst, inst.nsrc) {
                (1, 2) | (1, 1) | (0, 2) => encode_rrx(inst.opcode, dst, src1, inst.modbits, src2),
                (0, 1) => encode_label(inst.opcode, self.get_label_index(src2)?, ret.len()),
                (0, 0) => (inst.opcode as u32) << bits::OPCODE_OFF,
                (_, _) => panic!("Unsupported addressing mode for '{}'", inst.name),
            };
            ret.push(tmp);
        }
        Ok(ret)
    }

    fn get_label_index(&self, label_op: Operand) -> Result<usize, ErrKind> {
        if let Operand::Label(ident) = label_op {
            if let Some(&label_at) = self.labels.get(&ident) {
                Ok(label_at)
            } else {
                Err(ErrKind::UndefinedLabel(ident))
            }
        } else {
            panic!("Non-label operand passed to get_label_pos");
        }
    }

    fn next_tok(&mut self) -> Result<Token, ErrKind> {
        while let Some(c) = self.scn.peek() {
            if c == '\t' || c == ' ' {
                self.scn.next();
                continue;
            }
            if c == '@' || c == '/' {
                eat_comment(&mut self.scn)?;
                continue;
            }

            return match c {
                '+' | '-' | '0'..='9' => immediate(&mut self.scn),
                c if is_ident_char(c) => identifier(&mut self.scn),
                c => {
                    self.scn.next();
                    Ok(Token::Char(c))
                }
            };
        }
        Ok(Token::Eof)
    }

    fn make_statement(&mut self, inst: Instruction) -> Result<Statement, ErrKind> {
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
                _ => return Err(ErrKind::OperandExp),
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
                _ => return Err(ErrKind::OperandExp),
            };
        }
        // If operand is not immediate and modifier is present, then error
        if let Operand::Imm(_) = src2 {
            // Operand is immediate, fine
        } else if inst.modbits != bits::MOD_DEF {
            return Err(ErrKind::IllegalModifier);
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

/// Encodes the format `inst reg, reg, reg|imm`
fn encode_rrx(opcode: u8, dst: u8, src1: u8, modbits: u8, src2: Operand) -> u32 {
    match src2 {
        Operand::Reg(regs2) => {
            (opcode as u32) << bits::OPCODE_OFF
                | (dst as u32) << bits::DST_OFF
                | (src1 as u32) << bits::SRC1_OFF
                | (regs2 as u32) << bits::SRC2_OFF
        }
        Operand::Imm(imm) => {
            (opcode as u32) << bits::OPCODE_OFF
                | 1 << bits::IMMBIT_OFF
                | (dst as u32) << bits::DST_OFF
                | (src1 as u32) << bits::SRC1_OFF
                | (modbits as u32) << bits::MOD_OFF
                | (imm as u32)
        }
        Operand::Label(_) => panic!("This function cannot encode Operand::Label types"),
    }
}

/// Encodes the format `inst label`
fn encode_label(opcode: u8, label_at: usize, cur_at: usize) -> u32 {
    let offset = (label_at as i32 - cur_at as i32) as u32;
    (opcode as u32) << bits::OPCODE_OFF | (offset & (!0u32 >> bits::OPCODE_BITS))
}

/// := '@' [^'\n']* '\n'
///  | "/*" [^"*/"]* "*/"
fn eat_comment(scn: &mut Scanner) -> Result<(), ErrKind> {
    if scn.eat_prefix("@") {
        scn.take_while(|c| c != '\n');
        Ok(())
    } else if scn.eat_prefix("/*") {
        while !scn.eat_prefix("*/") {
            if scn.next() == None {
                return Err(ErrKind::OpenComment);
            }
        }
        Ok(())
    } else {
        // We know that the first char is '/', so '*' is missing
        Err(ErrKind::CharExp('*'))
    }
}

fn immediate(scn: &mut Scanner) -> Result<Token, ErrKind> {
    let mut base = 10;
    let mut is_neg = false;
    let num: u16;

    if let Some(c) = scn.peek() {
        if c == '+' || c == '-' {
            scn.next();
            is_neg = c == '-';
        }
    }
    if let Some('0') = scn.peek() {
        match scn.peekn(1) {
            Some('x') => base = 16,
            Some('o') => base = 8,
            Some('b') => base = 2,
            _ => {}
        }
        if base != 10 {
            scn.next();
            scn.next();
        }
    }

    let num_str = scn.take_while(|c| c.is_ascii_alphanumeric());
    match u16::from_str_radix(num_str, base) {
        Ok(ntmp) => {
            num = ntmp;
        }
        Err(e) => match e.kind() {
            IntErrorKind::PosOverflow => return Err(ErrKind::ImmOverflow),
            _ => return Err(ErrKind::InvalidImm),
        },
    }
    if is_neg {
        // Check for overflow and then convert to 2's Complement representation
        if num > std::i16::MIN.unsigned_abs() {
            return Err(ErrKind::ImmOverflow);
        }
        return Ok(Token::Imm(!num + 1));
    }
    Ok(Token::Imm(num))
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '$')
}

fn identifier(citer: &mut Scanner) -> Result<Token, ErrKind> {
    let ident = citer.take_while(is_ident_char);
    // Can be a register name or instruction name
    if let Some(&reg) = REGISTERS.iter().find(|&&reg| reg.0 == ident) {
        return Ok(Token::Reg(reg.1));
    }
    match instruction(&ident) {
        Ok(Some(tok)) => return Ok(tok),
        Ok(None) => {}
        Err(e) => return Err(e),
    }
    Ok(Token::Ident(String::from(ident)))
}

fn instruction(mut instr: &str) -> Result<Option<Token>, ErrKind> {
    let modbits: u8;

    if instr.ends_with('u') {
        instr = instr.strip_suffix('u').unwrap();
        modbits = bits::MOD_U;
    } else if instr.ends_with('h') {
        instr = instr.strip_suffix('h').unwrap();
        modbits = bits::MOD_H;
    } else {
        modbits = bits::MOD_DEF;
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
        if modbits != bits::MOD_DEF && !info::supports_mod(opcode) {
            return Err(ErrKind::IllegalModifier);
        }

        return Ok(Some(Token::Inst(Instruction {
            name,
            opcode,
            ndst,
            nsrc,
            modbits,
        })));
    }

    Ok(None)
}

pub fn parse_code(input: &str) -> Result<Vec<u32>, ParseErr> {
    let mut asm = Parser::new(input);
    match asm.parse() {
        Ok(ret) => Ok(ret),
        Err(kind) => Err(ParseErr {
            line: asm.line_num(),
            kind,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        bits, immediate, instruction, opcodes, ErrKind, Instruction, Parser, Scanner, Token,
    };
    use crate::parser::parse_code;

    #[test]
    fn test_scanner() {
        let mut s = Scanner::new("1234YYXXX");
        assert_eq!(s.peek(), Some('1'));
        assert_eq!(s.take_while(|ch| ch.is_numeric()), "1234");
        assert_eq!(s.take_while(|ch| ch == 'Y'), "YY");
        assert_eq!(s.take_while(|ch| ch == 'X'), "XXX");
    }

    #[test]
    fn imm_test() {
        let test_pairs: [(&str, Result<Token, ErrKind>); 6] = [
            ("42", Ok(Token::Imm(42))),
            ("0", Ok(Token::Imm(0))),
            ("-42", Ok(Token::Imm(!42 + 1))),
            ("-0x69", Ok(Token::Imm(!0x69 + 1))),
            ("0x1FFFF", Err(ErrKind::ImmOverflow)),
            ("0x1oops", Err(ErrKind::InvalidImm)),
        ];
        for (test, res) in test_pairs {
            assert_eq!(immediate(&mut Scanner::new(test)), res);
        }
    }

    #[test]
    fn instruction_test() {
        assert_eq!(
            instruction("add"),
            Ok(Some(Token::Inst(Instruction {
                name: "add",
                opcode: opcodes::ADD,
                ndst: 1,
                nsrc: 2,
                modbits: bits::MOD_DEF,
            })))
        );
        assert_eq!(
            instruction("addh"),
            Ok(Some(Token::Inst(Instruction {
                name: "add",
                opcode: opcodes::ADD,
                ndst: 1,
                nsrc: 2,
                modbits: bits::MOD_H,
            })))
        );
        // Invalid ones
        assert_eq!(instruction("nopu"), Err(ErrKind::IllegalModifier));
        assert_eq!(instruction("nosuchins"), Ok(None));
    }

    #[test]
    fn test_fine() {
        // Test only for first instruction
        let test_pairs: [(&str, u32); 4] = [
            ("mov r0, -0x1\n", 0b01001_1_0000_0000_00_1111111111111111),
            ("add r0, r1, r2\n", 0b00000_0_0000_0001_0010 << 14),
            (
                "add r0, r1, /* Block comment */ 0b1101\n",
                0b00000_1_0000_0001_00_0000000000001101,
            ),
            (
                "b has_label\n ret\n ret\n has_label: ret\n",
                0b10010_000000000000000000000000011,
            ),
        ];
        for (input, res) in test_pairs {
            assert_eq!(Parser::new(input).parse().unwrap()[0], res);
        }
    }

    #[test]
    fn test_bad() {
        let test_pairs: [(&str, ErrKind); 12] = [
            ("add r0, r1", ErrKind::CharExp(',')),
            ("add r0, /* uncomp*", ErrKind::OpenComment),
            ("/ *Illegal comment */", ErrKind::CharExp('*')),
            ("add r0, r1, r4", ErrKind::CharExp('\n')),
            ("add r0, r1, \n", ErrKind::OperandExp),
            ("addh r0, r1, r2 \n", ErrKind::IllegalModifier),
            ("noph\n", ErrKind::IllegalModifier),
            ("b r0\n", ErrKind::IdentExp),
            ("cmp 24, 88\n", ErrKind::RegExp),
            ("r13 add r11\n", ErrKind::IllegalToken),
            (
                "b undefme\n",
                ErrKind::UndefinedLabel(String::from("undefme")),
            ),
            (
                "abc:\n\n abc: ret\n",
                ErrKind::DuplicateLabel(String::from("abc")),
            ),
        ];
        for (input, err) in test_pairs {
            assert_eq!(parse_code(input).unwrap_err().kind, err);
        }
    }
}

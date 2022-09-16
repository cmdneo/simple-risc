use simple_risc::{
    emulator::Emulator,
    parser::{ParseErr, Parser},
};

#[test]
fn test_fine() {
    // Test only for first instruction
    let test_pairs: [(&str, u32); 4] = [
        ("mov r0, -0x1\n", 0b01001_1_0000_0000_00_1111111111111111),
        ("add r0, r1, r2\n", 0b00000_0_0000_0001_0010 << 14),
        (
            "add r0, r1, 0b1101\n",
            0b00000_1_0000_0001_00_0000000000001101,
        ),
        (
            "b has_label\n ret\n ret\n has_label: ret\n",
            0b10010_000000000000000000000000011,
        ),
    ];
    for (input, res) in test_pairs {
        assert_eq!(Parser::from(input).parse().unwrap()[0], res);
    }
}

#[test]
fn test_bad() {
    let test_pairs: [(&str, ParseErr); 10] = [
        ("add r0, r1", ParseErr::CharExpected(',')),
        ("add r0, r1, r4", ParseErr::CharExpected('\n')),
        ("add r0, r1, \n", ParseErr::OperandExpected),
        ("addh r0, r1, r2 \n", ParseErr::IllegalModifier),
        ("noph\n", ParseErr::IllegalModifier),
        ("b r0\n", ParseErr::IdentifierExpected),
        ("cmp 24, 88\n", ParseErr::RegisterExpected),
        ("r13 add r11\n", ParseErr::UnexpectedToken),
        (
            "b undefme\n",
            ParseErr::UndefinedLabel(String::from("undefme")),
        ),
        (
            "abc:\n\n abc: ret\n",
            ParseErr::DuplicateLabel(String::from("abc")),
        ),
    ];
    for (input, err) in test_pairs {
        assert_eq!(Parser::from(input).parse().unwrap_err(), err);
    }
}

#[test]
fn test_factorial() {
    let code = "@ Recursive Factorial function test
    b main
    main:
        mov sp, 1024 @ Stack top
        mov r0, 1     @ Result
        mov r1, 5     @ N = 5
        call factorial
        call exit

    factorial:
        sub sp, sp, 4  @Stack create 4 bytes
        st r15, 0[sp]   @Save return address
        mul r0, r0, r1
        sub r1, r1, 1
        cmp r1, 1
        beq ret_last
        call factorial
    ret_last:
        ld r15, 0[sp]
        add sp, sp, 4 @Stack destroy 4 bytes
        ret
        
    exit: @ Nothing
    ";
    let bincode = Parser::from(code).parse().unwrap();
    let mut emul = Emulator::from(&bincode);
    emul.exec().unwrap();
    assert_eq!(emul.get_reg_val(0), 120);
}

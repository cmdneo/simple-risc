use simple_risc::{emulator::Emulator, parser::parse_and_assemble};

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
    let bincode = parse_and_assemble(code).unwrap();
    let mut emul = Emulator::new(&bincode);
    emul.exec().unwrap();
    assert_eq!(emul.get_reg_val(0), 120);
}

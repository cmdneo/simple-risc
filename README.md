SimpleRISC
=====
Implementation of *SimpleRISC* emulator as desrcibed in the book [Basic Computer Architecture](https://www.cse.iitd.ac.in/~srsarangi/archbooksoft.html)
using **The Rust Programming Language**.  

### Building and running the tests

    $ cargo build
    $ cargo test

The programs starts executing from the first instruction present in the file and stops when the program-counter(`pc`) becomes invalid.  
Valid values for `pc` lie in the interval `[0, TOTAL_INSTRUCTION_COUNT)`.

See [simpleRISC.md](simpleRISC.md) for information about instructions and their semantics.

Examples
---

Program for finding factorial:
```
@ After its execution r0 will have the value 120
    mov r0, 1 @ Store result in
    mov r1, 5 @ N = 5

loop:
    mul r0, r0, r1
    sub r1, r1, 1
    cmp r1, 0
    bgt loop
```

Recursive version of the factorial program:
```
    b main

main:
    mov sp, 1024    @ Set Stack top
    @ r0 will have value 720 when the program ends
    mov r0, 1       
    mov r1, 6       @ N = 6
    call factorial
    b exit

factorial:
    sub sp, sp, 4   @ Stack alloc 4 bytes
    st r15, 0[sp]   @ Save return address
    mul r0, r0, r1
    sub r1, r1, 1
    cmp r1, 1
    beq ret_fac
    call factorial
ret_fac:
    ld r15, 0[sp]   @ Restore return address
    add sp, sp, 4   @ Stack free 4 bytes
    ret

exit: @ End
```

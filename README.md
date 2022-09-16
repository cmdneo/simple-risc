SimpleRISC
=====
Implementation of *SimpleRISC* emulator as desrcibed in
*Chapter 3* of the book [Basic Computer Architecture](https://www.cse.iitd.ac.in/~srsarangi/archbooksoft.html)
using **The Rust Programming Language** with some modifications.

### Building and running the tests

    $ cargo build
    $ cargo test

See [simpleRISC.md](simpleRISC.md) for information about instructions and their semantics.

Examples
---

Program for finding the factorial of 5
```
@ After its execution r0 will have the value 120
    mov r0, 1 @ Store result in
    mov r1, 5 @ Calculate factorial of 5

loop:
    mul r0, r0, r1
    sub r1, r1, 1
    cmp r1, 0
    bgt loop
```
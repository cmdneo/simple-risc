SimpleRISC
===

Taken from *Chapter 3* of the book [Basic Computer Architecture](https://www.cse.iitd.ac.in/~srsarangi/archbooksoft.html).

All calculations use **32 bit 2's complement wrap-around arithmetic**.  
The program is executed from top to bottom.

Syntax
---
### Statement
```
[label_name:] [instruction operands] [@ Inline comment] '\n'
```
Each statement must be terminated by a newline.
Operands are seperate by comma as: `op1, op2`


### Immediate
And immediate is an integer value of 16-bits.  
The value must lie in the interval `[-32768, 65535]`.  
Negative values are converted to the corresponding 2's complement representaion.

It starts with an optional sign followed by an
optional prefix(see table) for base followed by at least one valid digit.  
Default base is 10(obviously).  
No spaces are allowed in between.
| Prefix | Base |
| ------ | ---- |
| `0x`   | 16   |
| `0o`   | 8    |
| `0b`   | 2    |

### Identifiers(labels)
An identifier is a sequence of a alphanumeric, and `$`(dollar), `_`(underscore) and `.`(period) characters.  
The first character cannot be a digit.

Registers
---
Registers `r[0-15]` are directly accessible to the user.  
`sp` is an alias for `r14` which is generally used as a stack pointer.  
`r15` is used as return address register by `call` and `ret` instructions.

`flags` register used for storing result of the `cmp` instruction.  
It has two fields `flags.E` and `flags.GT`.

The program counter `pc` stores index of the instruction being executed.  
Valid values for `pc` lie in the interval `[0, TOTAL_INSTRUCTION_COUNT)`  
If `pc` becomes invalid then the program execution stops normally.

Registers `flags` and `pc` are not directly accesible to the user.


Instructions
---
First operand is denoted by `A`, second by `B` and third by `C`.  
First operand is always the destination register(except for `st` instruction).  
For `ld` and `st` instructions effective memory address must be aligned to 4 bytes.  
Only the lower 5 bits of the third operand are considered for
shift instructions(`lsl`, `lsr` and `asr`).

| Format                  | Action                                     |
| ----------------------- | ------------------------------------------ |
| `mov reg, reg/imm`      | `A <- B`                                   |
| `add reg, reg, reg/imm` | `A <- B + C`                               |
| `sub reg, reg, reg/imm` | `A <- B - C`                               |
| `mul reg, reg, reg/imm` | `A <- B * C`                               |
| `div reg, reg, reg/imm` | `A <- B / C`                               |
| `mod reg, reg, reg/imm` | `A <- B % C`                               |
| `cmp reg, reg/imm`      | `flags.E <- A == B`<br>`flags.GT <- A > B` |
| `and reg, reg, reg/imm` | `A <- B & C`                               |
| `or reg, reg, reg/imm`  | `A <- B \| C`                              |
| `not reg, reg/imm`      | `A <- !B`                                  |
| `lsl reg, reg, reg/imm` | `A <- B << C`                              |
| `lsr reg, reg, reg/imm` | `A <- B >> C`                              |
| `asr reg, reg, reg/imm` | `A <- B >>> C`    [^1]                     |
| `ld reg, imm[reg]`      | `A <- [B + C]`   (`imm` is `C`)            |
| `st reg, imm[reg]`      | `[B + C] <- A`   (`imm` is `C`) [^2]       |
| `b label`               | Unconditional branch                       |
| `beq label`             | If `flags.E` set, then branch.             |
| `bgt label`             | If `flags.GT` set, then branch             |
| `call label`            | `r15 <- (pc + 1)`, then branch             |
| `ret`                   | `pc <- r15` (branches to return address)   |
| `nop`                   | No operation                               |
| `sys`                   | `r0 <- syscall(r0)` [^3]                   |


### Modifiers
Modifiers can be used with the following instructions `add`, `sub`, `mul`, `div`, `mod`, `cmp`, `and`, `or`, `not` and `mov`.  
A modifier can only be used when some source operand is an immediate.

Modifiers affect the way in which immediate is expaded from 16 to 32 bits inside the cpu:  
No modifier : Sign extension is performed  
Modifier `u`  : Immediate is treated as unsigned(loaded as it is)  
Modifier `h`  : Immediate is loaded into the higher 16-bits of the register.  

A modifier is suffixed to an instruction, like `addh`, `subu` ...

Extras
---
Some (maybe)useful extensions to **simpleRISC**

### The `sys` instruction

The value returned from the syscall is stored in `r0`.  
The `r0` contains the syscall number.  
Arguments(max 4) as passed via registers as: 
| Argument | Register |
| :------: | :------: |
|  Arg-1   |   `r1`   |
|  Arg-2   |   `r2`   |
|  Arg-3   |   `r3`   |
|  Arg-4   |   `r4`   |

Rest of the registers are preserved.

### List of syscalls

(0) `getchar()`: Reads one byte and returns it, returns -1 on failure  
(1) `putchar(byte c)`: Prints the character and returns it, returns -1 on failure

[^1]: Arithmetic Right shift.

[^2]: Exception, here the destination register acts as a source

[^3]: This instruction([See sys instruction](#the-sys-instruction)) is an extension for better interaction with the language.

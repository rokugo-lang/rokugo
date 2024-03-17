# Virtual registers in Rokugo's IR
IR Edition: 0

In IR context word `register` refers to virtual registers.

## General-purpose registers
IR have 15 general-purpose virtual registers where each accept: 64, 32, 16, 8 bit naturals or integers, or platform-sized pointer.

Each register can store only one value at the same time. When a register is written in lower bits mode e.g. 32 bits, the top of register are undefined value. `TODO: make sure it is needed`

This registers is named with prefix `X`, then names will be `X0`-`X14`. They also have subfix with their mode:
- `P` - for pointer
- `Q` - for 64-bit value
- `D` - for 32-bit value
- `W` - for 16-bit value
- `B` - for 8-bit value

What create name like: `X3Q`.

### Grouping
For better optimalization, and easier JIT compilation for some native architectures, IR provide general-purpose virtual register groups what lint JIT compiler which register should be used.

## Special registers
Some of IR instructions operate on special virtual registers.

### Stack pointer
Stack pointer (`SP`) virtual register. Holds the last used memory address of a top of thread stack as platform-sized pointer. It can be offseted only by few instructions.

## Edition explanation
Current edition uses only 15 general-purpose registers, because AMD64 have 16 general-purpose register where one of them is stack pointer (`SP`), which is special in IR.



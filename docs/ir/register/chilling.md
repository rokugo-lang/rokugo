# Virtual register chilling
Rokugo's IR contains almost unlimited amount of virtual registers. Almost, because of memory requirements it is for now 1024 for each type. Native architectures due to physical, and cost related logical limits do not have such much registers.

Runtime executors like JIT compiler, must map virtual registers into native, what cost computing power, and slows JIT compilation. To optimize that case IR indroctuses virtual register chilling. This is a simple hint included in IR which explicit informs runtime executor which registers relativly from current instruction will be needed latest.

## Intended runtime use
Depending on the implementation, during register allocation JIT compiler veryfies if any register stay still free. If not it uses this hint for temporaly store needed registers into e.g. stack memory, what make register free.

In next instructions, where previous value is needed, then JIT compiler restoring that value from their temporaly storage, and if necessary chill another registers.

## Data
Whenever hint for chilling passes information about each type virtual registers. This is caused by registers sharing in native architectures. For example AMD64 share `Nat32` register with `Nat64` type, then for allocate next `Nat32` register, `Nat64` must be chilled.

## Impact on compilation time
This hint is computed in AOT, what only takes publisher compilation time, and do not affect end users of program.

> Impact caused by reading additional data etc. was omitted.

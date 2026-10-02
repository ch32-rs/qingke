#!/bin/bash

set -ex

cargo publish  --target riscv32imc-unknown-none-elf --allow-dirty

(cd qingke-rt/macros && cargo publish --target riscv32imc-unknown-none-elf --allow-dirty)

test -f qingke-rt/build/cfgs.inc.rs
(cd qingke-rt && cargo publish --target riscv32imc-unknown-none-elf --allow-dirty --no-verify)

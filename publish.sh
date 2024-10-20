#!/bin/bash

set -ex

cargo publish  --target riscv32imac-unknown-none-elf --allow-dirty

(cd qingke-rt/macros && cargo publish --target riscv32imac-unknown-none-elf --allow-dirty)

(cd qingke-rt && cargo publish --target riscv32imac-unknown-none-elf --allow-dirty)

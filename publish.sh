#!/bin/bash

set -ex

cargo publish  --target riscv32imac-unknown-none-elf

(cd qingke-rt/macros && cargo publish --target riscv32imac-unknown-none-elf)

(cd qingke-rt && cargo publish --target riscv32imac-unknown-none-elf)

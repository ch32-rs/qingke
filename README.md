# qingke & qingke-rt

[![Crates.io][badge-license]][crates]
[![Crates.io][badge-version]][crates]
[![docs.rs][badge-docsrs]][docsrs]

[badge-license]: https://img.shields.io/crates/l/qingke?style=for-the-badge
[badge-version]: https://img.shields.io/crates/v/qingke?style=for-the-badge
[badge-docsrs]: https://img.shields.io/docsrs/qingke?style=for-the-badge
[crates]: https://crates.io/crates/qingke
[docsrs]: https://docs.rs/qingke

Low level access to WCH's QingKe RISC-V processors.

## QingKe V3V (CH32V407 / CH32V467)

Select the `v3v` leaf feature in both `qingke` and `qingke-rt`, rather than
`v4f`. A CH32V467WEU6 reports `marchid=0xdc68d876` (WCH V3V),
`mimpid=0xdc688003`, and `misa=0x40b01107` (RV32, ABCIMUVX; no F).
These are observed identity values, not proof of safe atomic instructions or
ratified vector/bit-manipulation encodings. Start with the soft-float
`riscv32imc-unknown-none-elf` target; selecting this feature does not enable
compiler vector code generation or bypass the atomics guard.

The runtime follows WCH's
[V4x7 startup](https://github.com/openwch/ch32v407_ch32v467/blob/main/EVT/EXAM/SRC/Startup/startup_ch32v4x7.S)
for vector control, prefetch, nesting and hardware stacking. It initializes
VS (not FS) in `mstatus`, stays in Machine mode by default, and uses the SDK's
User-mode value with `u-mode`. The vector table must be 1 KiB aligned, including
with `highcode`. Vector instruction use and vector context handling in ISRs
are not validated by this feature. `qingke::dm` is intentionally unavailable:
the V3V SDI data address has not been validated; use RTT or UART for logging.

## qingke-rt

This crate provides the runtime support for QingKe RISC-V processors.

This provides `riscv`/`riscv-rt` like functionality, with the following differences:

- Use vector table for interrupt handling
- Handle 1KB address alignment for the entry point(Qingke V2)
- In-SRAM code executing, `highcode` handling
- PFIC support
- **Conflicts with `riscv-rt` crate**

## Usage

```rust
#[qingke_rt::entry]
fn main() -> ! {
    loop {}
}

// Or if you are using the embassy framework
#[embassy_executor::main(entry = "qingke_rt::entry")]
async fn main(spawner: Spawner) -> ! { ... }

#[qingke_rt::interrupt]
fn UART0() {
    // ...
}

#[qingke_rt::highcode]
fn some_highcode_fn() {
    // ...
    // This fn will be loaded into the highcode(SRAM) section.
    // This is required for BLE, recommended for interrupt handles.
}
```

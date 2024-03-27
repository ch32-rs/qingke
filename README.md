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

## qingke-rt

This crate provides the runtime support for QingKe RISC-V processors.

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

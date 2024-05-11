# qingke-rt

Replaces `ch32v-rt` as the name is not suitable for publishing.

QingKe is the name of the RISC-V core.

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

// Interrupt provided by the IP core (not peripherals)
#[qingke_rt::interrupt(core)]
fn SysTick() {
    // ...
}

#[qingke_rt::highcode]
fn some_highcode_fn() {
    // ...
    // This fn will be loaded into the highcode(SRAM) section.
    // This is required for BLE, recommended for interrupt handles.
}
```

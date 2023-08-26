# embassy-time-rp2040

[![](https://badgers.space/crates/info/embassy-time-rp2040)](https://crates.io/crates/embassy-time-rp2040)
[![](https://badgers.space/github/open-issues/c-h-johnson/embassy-time-rp2040)](https://github.com/c-h-johnson/embassy-time-rp2040/issues)

An embassy-time driver using rp2040-hal for embedded-hal compatibility

# Purpose

See the
[embassy-time README](https://github.com/embassy-rs/embassy/blob/main/embassy-time/README.md#time-driver)
and the
[embassy-time documentation](https://docs.rs/embassy-time/latest/embassy_time/driver/index.html)
for an explanation.

# Example

```rust
#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;

use rp_pico as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    timer::Timer,
    watchdog::Watchdog,
};

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    embassy_time_rp2040::init(timer);

    // ...
}
```

#![no_main]
#![no_std]

use embedded_hal::blocking::delay::DelayMs;
use nrf52840_hal::{self as hal, Temp, Timer};

use airlog as _; // global logger + panicking-behavior + memory layout

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    let board = hal::pac::Peripherals::take().unwrap();
    let mut temp = Temp::new(board.TEMP);

    let mut timer = Timer::new(board.TIMER0);

    loop {
        let temperature: f32 = temp.measure().to_num();
        defmt::info!("{=f32} °C", temperature);
        timer.delay_ms(60 * 1000u32);
    }
}

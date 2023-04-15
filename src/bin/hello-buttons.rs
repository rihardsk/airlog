#![no_main]
#![no_std]

use embedded_hal::digital::v2::OutputPin;
use nrf52840_hal::{
    self as hal,
    gpio::{p0::Parts as P0Parts, Level},
};

use airlog::{self as _, peripherals::Button}; // global logger + panicking-behavior + memory layout

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    let board = hal::pac::Peripherals::take().unwrap();
    // let mut timer = Timer::new(board.TIMER0);
    let pins = P0Parts::new(board.P0);

    let button = Button::new(pins.p0_11.degrade());
    let mut led_1 = pins.p0_13.into_push_pull_output(Level::High);

    loop {
        if button.is_pressed() {
            led_1.set_low().unwrap();
        } else {
            led_1.set_high().unwrap();
        }
    }
}

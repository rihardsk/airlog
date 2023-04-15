#![no_main]
#![no_std]

use embedded_hal::{blocking::delay::DelayMs, digital::v2::OutputPin};
use nrf52840_hal::{
    self as hal,
    gpio::{p0::Parts as P0Parts, Level},
    Timer,
};

use airlog as _; // global logger + panicking-behavior + memory layout

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    let board = hal::pac::Peripherals::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let pins = P0Parts::new(board.P0);
    // LEDs are active low
    let mut led_1 = pins.p0_13.into_push_pull_output(Level::High);
    let mut led_2 = pins.p0_14.into_push_pull_output(Level::High);
    let mut led_3 = pins.p0_15.into_push_pull_output(Level::High);
    let mut led_4 = pins.p0_16.into_push_pull_output(Level::High);

    for led_states in (1u8..16u8).cycle() {
        let led_1_state = led_states & 0b0001 > 0;
        let led_2_state = led_states & 0b0010 > 0;
        let led_3_state = led_states & 0b0100 > 0;
        let led_4_state = led_states & 0b1000 > 0;

        if led_1_state {
            led_1.set_low().unwrap();
        } else {
            led_1.set_high().unwrap();
        }
        if led_2_state {
            led_2.set_low().unwrap();
        } else {
            led_2.set_high().unwrap();
        }
        if led_3_state {
            led_3.set_low().unwrap();
        } else {
            led_3.set_high().unwrap();
        }
        if led_4_state {
            led_4.set_low().unwrap();
        } else {
            led_4.set_high().unwrap();
        }
        timer.delay_ms(1000u32);
    }

    airlog::exit()
}

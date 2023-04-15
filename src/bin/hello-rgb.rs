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
    // We're using a common anode RGB LED, so it's active low
    let mut led_r = pins.p0_03.into_push_pull_output(Level::High);
    let mut led_g = pins.p0_04.into_push_pull_output(Level::High);
    let mut led_b = pins.p0_28.into_push_pull_output(Level::High);

    for led_states in (1u8..8u8).cycle() {
        let led_r_state = led_states & 0b0001 > 0;
        let led_g_state = led_states & 0b0010 > 0;
        let led_b_state = led_states & 0b0100 > 0;

        if led_r_state {
            led_r.set_low().unwrap();
        } else {
            led_r.set_high().unwrap();
        }
        if led_g_state {
            led_g.set_low().unwrap();
        } else {
            led_g.set_high().unwrap();
        }
        if led_b_state {
            led_b.set_low().unwrap();
        } else {
            led_b.set_high().unwrap();
        }
        timer.delay_ms(1000u32);
    }

    airlog::exit()
}

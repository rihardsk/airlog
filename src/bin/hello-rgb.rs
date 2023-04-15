#![no_main]
#![no_std]

use embedded_hal::{blocking::delay::DelayMs, digital::v2::OutputPin};
use hal::gpio::{Output, Pin, PushPull};
use nrf52840_hal::{
    self as hal,
    gpio::{p0::Parts as P0Parts, Level},
    Timer,
};

use airlog as _; // global logger + panicking-behavior + memory layout

struct LEDControl {
    r: Pin<Output<PushPull>>,
    g: Pin<Output<PushPull>>,
    b: Pin<Output<PushPull>>,
}

impl LEDControl {
    pub fn new<Mode>(led_red: Pin<Mode>, led_green: Pin<Mode>, led_blue: Pin<Mode>) -> Self {
        LEDControl {
            r: led_red.into_push_pull_output(Level::High),
            g: led_green.into_push_pull_output(Level::High),
            b: led_blue.into_push_pull_output(Level::High),
        }
    }

    pub fn set_state(&mut self, state_red: bool, state_green: bool, state_blue: bool) {
        if state_red {
            self.r.set_low().unwrap();
        } else {
            self.r.set_high().unwrap();
        }
        if state_green {
            self.g.set_low().unwrap();
        } else {
            self.g.set_high().unwrap();
        }
        if state_blue {
            self.b.set_low().unwrap();
        } else {
            self.b.set_high().unwrap();
        }
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    let board = hal::pac::Peripherals::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let pins = P0Parts::new(board.P0);
    // We're using a common anode RGB LED, so it's active low
    let led_r = pins.p0_03.degrade();
    let led_g = pins.p0_04.degrade();
    let led_b = pins.p0_28.degrade();

    let mut led = LEDControl::new(led_r, led_g, led_b);

    for led_states in (1u8..8u8).cycle() {
        let led_r_state = led_states & 0b0001 > 0;
        let led_g_state = led_states & 0b0010 > 0;
        let led_b_state = led_states & 0b0100 > 0;

        led.set_state(led_r_state, led_g_state, led_b_state);
        timer.delay_ms(1000u32);
    }

    airlog::exit()
}

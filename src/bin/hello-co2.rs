#![no_main]
#![no_std]

use cortex_m::prelude::{_embedded_hal_blocking_delay_DelayMs, _embedded_hal_timer_CountDown};
use embedded_hal::blocking::i2c;
use hal::{twim, Temp, Twim};
use nrf52840_hal::{self as hal, gpio::p0::Parts as P0Parts, Timer};

use airlog::{
    self as _,
    peripherals::{scd30, Button, LEDControl, SCD30},
}; // global logger + panicking-behavior + memory layout

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    let board = hal::pac::Peripherals::take().unwrap();
    let pins = P0Parts::new(board.P0);
    let mut temp = Temp::new(board.TEMP);

    let mut periodic_timer = Timer::periodic(board.TIMER0);

    let pin_r = pins.p0_03.degrade();
    let pin_g = pins.p0_04.degrade();
    let pin_b = pins.p0_28.degrade();
    let mut led = LEDControl::new(pin_r, pin_g, pin_b);

    let scl = pins.p0_30.into_floating_input().degrade();
    let sda = pins.p0_31.into_floating_input().degrade();
    let twim_pins = twim::Pins { scl, sda };
    let i2c = Twim::new(board.TWIM0, twim_pins, twim::Frequency::K100);

    let mut scd30 = SCD30::new(i2c);
    let version = scd30.get_firmware_version().unwrap();
    defmt::info!(
        "SCD30 firmware version: {=u8}.{=u8}",
        version.major,
        version.minor
    );

    scd30.start_continuous_measurement(1023).unwrap();

    loop {
        // periodic_timer.start(1000_u32);
        loop {
            if scd30.data_ready().unwrap() {
                break;
            }
        }
        let reading = scd30.read_measurement().unwrap();
        if reading.co2 < 1000_f32 {
            led.set_state(false, true, false);
        } else if reading.co2 < 1600_f32 {
            led.set_state(false, true, true);
        } else {
            led.set_state(true, false, false);
        }

        defmt::info!(
            "
            CO2 {=f32} ppm
            Temperature {=f32} °C
            Rel. humidity {=f32} %
        ",
            reading.co2,
            reading.temperature,
            reading.rel_humidity
        );
        periodic_timer.delay_ms(5000_u32);
    }
}

#![no_main]
#![no_std]

use cortex_m::prelude::{_embedded_hal_blocking_delay_DelayMs, _embedded_hal_timer_CountDown};
use embedded_hal::blocking::i2c;
use hal::{
    gpio::Level,
    pwm::{self, Pwm},
    twim, Temp, Twim,
};
use nrf52840_hal::{self as hal, gpio::p0::Parts as P0Parts, Timer};

use airlog::{
    self as _, logic,
    peripherals::{scd30, Button, LEDControl, PwmLEDControl, SCD30},
}; // global logger + panicking-behavior + memory layout

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    let board = hal::pac::Peripherals::take().unwrap();
    let pins = P0Parts::new(board.P0);
    let mut temp = Temp::new(board.TEMP);

    let mut periodic_timer = Timer::periodic(board.TIMER0);

    let pwm = Pwm::new(board.PWM0);

    let pin_r = pins.p0_03.into_push_pull_output(Level::High).degrade();
    let pin_b = pins.p0_04.into_push_pull_output(Level::High).degrade();
    let pin_g = pins.p0_28.into_push_pull_output(Level::High).degrade();

    pwm.set_output_pin(pwm::Channel::C0, pin_r);
    pwm.set_output_pin(pwm::Channel::C1, pin_g);
    pwm.set_output_pin(pwm::Channel::C2, pin_b);

    let mut led = PwmLEDControl::new(pwm);

    led.set_color(255, 0, 0);
    periodic_timer.delay_ms(300_u32);
    led.set_color(0, 255, 0);
    periodic_timer.delay_ms(300_u32);
    led.set_color(0, 0, 255);
    periodic_timer.delay_ms(300_u32);
    led.set_color(0, 0, 0);
    periodic_timer.delay_ms(300_u32);

    let scl = pins.p0_30.into_floating_input().degrade();
    let sda = pins.p0_31.into_floating_input().degrade();
    let twim_pins = twim::Pins { scl, sda };
    let i2c = Twim::new(board.TWIM0, twim_pins, twim::Frequency::K100);

    defmt::info!("Setting up SCD30");

    let mut scd30 = SCD30::new(i2c);
    let version = scd30.get_firmware_version().unwrap();
    defmt::info!(
        "SCD30 firmware version: {=u8}.{=u8}",
        version.major,
        version.minor
    );

    for i in 0..=100 {
        let fraction = i as f32 / 100.;
        let (r, g, b) = logic::colormap::smart_map_rgb(fraction);
        led.set_color(r, g, b);
        periodic_timer.delay_ms(30_u32);
    }
    periodic_timer.delay_ms(100_u32);

    scd30.start_continuous_measurement(1023).unwrap();

    loop {
        // periodic_timer.start(1000_u32);
        loop {
            if scd30.data_ready().unwrap() {
                break;
            }
        }
        let reading = scd30.read_measurement().unwrap();
        // if reading.co2 < 1000_f32 {
        //     led.set_color(0, 255, 0);
        // } else if reading.co2 < 1600_f32 {
        //     led.set_color(255, 255, 0);
        // } else {
        //     led.set_color(255, 0, 0);
        // }

        // current baseline ppm is 424
        let fraction = (reading.co2 - 424.) / (3000 - 424) as f32;
        let fraction = fraction.max(0.);
        let (r, g, b) = logic::colormap::smart_map_rgb(fraction);
        led.set_color(r, g, b);

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

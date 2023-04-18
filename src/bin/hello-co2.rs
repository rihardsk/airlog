#![no_main]
#![no_std]

use cortex_m::prelude::{_embedded_hal_blocking_delay_DelayMs, _embedded_hal_timer_CountDown};
use embedded_hal::blocking::i2c;
use hal::{
    gpio::Level,
    prelude::OutputPin,
    pwm::{self, Pwm},
    twim, Temp, Twim,
};
use hd44780_driver::{Cursor, CursorBlink, Direction, Display, DisplayMode, HD44780};
use micromath::F32Ext;
use nrf52840_hal::{self as hal, gpio::p0::Parts as P0Parts, gpio::p1::Parts as P1Parts, Timer};

use airlog::{
    self as _, logic,
    peripherals::{scd30, Button, LEDControl, PwmLEDControl, SCD30},
}; // global logger + panicking-behavior + memory layout

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    let board = hal::pac::Peripherals::take().unwrap();
    let core_peripherals = hal::pac::CorePeripherals::take().unwrap();
    let pins_0 = P0Parts::new(board.P0);
    let pins_1 = P1Parts::new(board.P1);
    let mut temp = Temp::new(board.TEMP);

    let mut periodic_timer = Timer::periodic(board.TIMER0);
    // let lcd_timer = DurationTimer(Timer::one_shot(board.TIMER1));
    // let mut lcd_timer = Timer::one_shot(board.TIMER1);
    let mut lcd_timer = hal::Delay::new(core_peripherals.SYST);

    let pwm = Pwm::new(board.PWM0);

    let pin_r = pins_0.p0_03.into_push_pull_output(Level::High).degrade();
    let pin_b = pins_0.p0_04.into_push_pull_output(Level::High).degrade();
    let pin_g = pins_0.p0_28.into_push_pull_output(Level::High).degrade();

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

    let scl = pins_0.p0_30.into_floating_input().degrade();
    let sda = pins_0.p0_31.into_floating_input().degrade();
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

    let rs = pins_1.p1_10.into_push_pull_output(Level::Low);
    let en = pins_1.p1_11.into_push_pull_output(Level::Low);
    let d4 = pins_1.p1_12.into_push_pull_output(Level::Low);
    let d5 = pins_1.p1_13.into_push_pull_output(Level::Low);
    let d6 = pins_1.p1_14.into_push_pull_output(Level::Low);
    let d7 = pins_1.p1_15.into_push_pull_output(Level::Low);

    // let mut lcd = LCD1602::new(en, rs, d4, d5, d6, d7, lcd_timer).unwrap();
    let mut lcd = HD44780::new_4bit(rs, en, d4, d5, d6, d7, &mut lcd_timer).unwrap();

    // Unshift display and set cursor to 0
    lcd.reset(&mut lcd_timer).unwrap();

    // Clear existing characters
    lcd.clear(&mut lcd_timer).unwrap();

    // Display the following string
    lcd.write_str("Hello, world!", &mut lcd_timer).unwrap();

    // Move the cursor to the second line
    lcd.set_cursor_pos(40, &mut lcd_timer).unwrap();

    // Display the following string on the second line
    lcd.write_str("I'm on line 2!", &mut lcd_timer).unwrap();

    periodic_timer.delay_ms(500_u32);
    scd30.start_continuous_measurement(1023).unwrap();
    lcd.clear(&mut lcd_timer).unwrap();

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

        let mut lcd_text: heapless::String<128> = heapless::String::new();
        let co2_int = reading.co2.round() as u32;
        // Do some ghetto padding operations so that the LCD text doesn't jump
        // around all the time. TODO: refactor
        let co2_int_len = u32_len(co2_int);
        for _ in 0..(4 - co2_int_len) {
            lcd_text.push_str(" ").unwrap();
        }
        ufmt::uwrite!(lcd_text, "{} ppm", co2_int).unwrap();
        lcd.set_cursor_pos(0, &mut lcd_timer).unwrap();
        lcd.write_str(&lcd_text, &mut lcd_timer).unwrap();

        lcd.set_cursor_pos(40, &mut lcd_timer).unwrap();
        // TODO: Can't output °, because it's probably part of unicode, not
        // ascii, See if there's a workaround using the hd44780 font table
        let temp_text = format_float_measurement(reading.temperature, 2, "C");
        lcd.write_str(&temp_text, &mut lcd_timer).unwrap();

        lcd.shift_cursor(Direction::Right, &mut lcd_timer).unwrap();
        lcd.shift_cursor(Direction::Right, &mut lcd_timer).unwrap();
        let humidity_text = format_float_measurement(reading.rel_humidity, 2, "%");
        lcd.write_str(&humidity_text, &mut lcd_timer).unwrap();

        periodic_timer.delay_ms(5000_u32);
    }
}

fn u32_len(num: u32) -> u8 {
    let mut count = 0;
    let mut num = num;
    while num > 0 {
        num /= 10_u32;
        count += 1;
    }
    count
}

// TODO: Works only on positive values
fn format_float_measurement(value: f32, pad_main: u8, unit: &str) -> heapless::String<16> {
    let mut output: heapless::String<16> = heapless::String::new();
    let int_part = value.floor() as u32;

    let int_len = u32_len(int_part);
    for _ in 0..(pad_main - int_len) {
        output.push_str(" ").unwrap();
    }
    let frac_part = (value.fract() * 100.) as u32;
    let mut frac_text: heapless::String<2> = heapless::String::new();
    if frac_part < 10 {
        frac_text.push_str("0").unwrap();
    }
    ufmt::uwrite!(frac_text, "{}", frac_part).unwrap();
    ufmt::uwrite!(output, "{}.{} {}", int_part, frac_text.as_str(), unit).unwrap();

    output
}

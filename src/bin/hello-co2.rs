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
    peripherals::{scd30, Button, LEDControl, PwmLEDControl, SensorReading, SCD30},
};
use sgp40::Sgp40; // global logger + panicking-behavior + memory layout

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    let board = hal::pac::Peripherals::take().unwrap();
    let core_peripherals = hal::pac::CorePeripherals::take().unwrap();
    let pins_0 = P0Parts::new(board.P0);
    let pins_1 = P1Parts::new(board.P1);
    let mut temp = Temp::new(board.TEMP);

    let mut builtin_led_1 = pins_0.p0_13.into_push_pull_output(Level::High);

    let mut periodic_timer = Timer::periodic(board.TIMER0);
    // let lcd_timer = DurationTimer(Timer::one_shot(board.TIMER1));
    // let mut lcd_timer = Timer::one_shot(board.TIMER1);
    let mut lcd_timer = hal::Delay::new(core_peripherals.SYST);
    let sgp40_timer = Timer::one_shot(board.TIMER1);

    let pwm = Pwm::new(board.PWM0);

    let pin_r = pins_1.p1_08.into_push_pull_output(Level::High).degrade();
    let pin_b = pins_1.p1_07.into_push_pull_output(Level::High).degrade();
    let pin_g = pins_1.p1_06.into_push_pull_output(Level::High).degrade();

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

    let scl = pins_1.p1_04.into_floating_input().degrade();
    let sda = pins_1.p1_05.into_floating_input().degrade();
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

    defmt::info!("Initializing SGP40 VOC sensor");
    // TODO: share the previous i2c
    let sda2 = pins_1.p1_02.into_floating_input().degrade();
    let scl2 = pins_1.p1_03.into_floating_input().degrade();
    let twim_pins2 = twim::Pins {
        scl: scl2,
        sda: sda2,
    };
    let i2c2 = Twim::new(board.TWIM1, twim_pins2, twim::Frequency::K100);
    let mut sgp40 = Sgp40::new(i2c2, 0x59, sgp40_timer);

    defmt::info!("Calibrating SGP40");
    // Discard the first 45 samples as the algorithm is just warming up.
    for _ in 1..45 {
        sgp40.measure_voc_index().unwrap();
    }
    periodic_timer.start(1_000_000_u32);
    for _ in 0..5 {
        let voc_after_calibration = sgp40.measure_voc_index().unwrap();
        defmt::info!(
            "Done calibrating SGP40: VOC idx {=u16}",
            voc_after_calibration
        );
        nb::block!(periodic_timer.wait()).unwrap();
    }

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

    defmt::info!("Entering loop");
    let mut seconds: u32 = 0;
    let mut reading = SensorReading {
        co2: 0.,
        rel_humidity: 0.,
        temperature: 0.,
    };
    let mut voc_index: u16;
    let mut builtin_led_state = hal::prelude::PinState::Low;
    periodic_timer.start(1_000_000_u32);
    loop {
        // periodic_timer.start(1000_u32);
        if seconds % 3 == 0 {
            loop {
                if scd30.data_ready().unwrap() {
                    break;
                }
            }
            reading = scd30.read_measurement().unwrap();

            // current baseline ppm is 424
            let fraction = (reading.co2 - 424.) / (3000 - 424) as f32;
            let fraction = fraction.max(0.);
            let (r, g, b) = logic::colormap::smart_map_rgb(fraction);
            led.set_color(r, g, b);
        }

        // TODO: figure out the right measurement approach here. crate docs
        // indicate that we should be making measurements with 1Hz frequency,
        // but this also kinda works, must check the datasheet
        voc_index = sgp40.measure_voc_index().unwrap();

        if seconds % 5 == 0 {
            defmt::info!(
            "
                CO2 {=f32} ppm
                Temperature {=f32} °C
                Rel. humidity {=f32} %
                VOC idx {=u16}
            ",
                reading.co2,
                reading.temperature,
                reading.rel_humidity,
                voc_index
            );

            lcd.set_cursor_pos(0, &mut lcd_timer).unwrap();
            let co2_text = format_float_measurement(reading.co2, 4, 0, "ppm");
            lcd.write_str(&co2_text, &mut lcd_timer).unwrap();

            lcd.shift_cursor(Direction::Right, &mut lcd_timer).unwrap();
            // TODO: can we make u32 stuff generic?
            let voc_text = format_u32_measurement(voc_index as u32, 3, "voc");
            lcd.write_str(&voc_text, &mut lcd_timer).unwrap();

            lcd.set_cursor_pos(40, &mut lcd_timer).unwrap();
            // TODO: Can't output °, because it's probably part of unicode, not
            // ascii, See if there's a workaround using the hd44780 font table
            let temp_text = format_float_measurement(reading.temperature, 2, 2, "C");
            lcd.write_str(&temp_text, &mut lcd_timer).unwrap();

            lcd.shift_cursor(Direction::Right, &mut lcd_timer).unwrap();
            lcd.shift_cursor(Direction::Right, &mut lcd_timer).unwrap();
            let humidity_text = format_float_measurement(reading.rel_humidity, 2, 2, "%");
            lcd.write_str(&humidity_text, &mut lcd_timer).unwrap();
        }

        builtin_led_1.set_state(builtin_led_state).unwrap();
        builtin_led_state = toggle_pin_state(builtin_led_state);

        nb::block!(periodic_timer.wait()).unwrap();
        seconds = seconds.overflowing_add(1).0;
    }
}

fn toggle_pin_state(value: hal::prelude::PinState) -> hal::prelude::PinState {
    match value {
        hal::prelude::PinState::Low => hal::prelude::PinState::High,
        hal::prelude::PinState::High => hal::prelude::PinState::Low,
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

// TODO: Works only on positive values, precision must be <=4
fn format_u32_measurement(value: u32, pad_main: u8, unit: &str) -> heapless::String<16> {
    let mut output: heapless::String<16> = heapless::String::new();

    let int_len = u32_len(value);
    for _ in 0..(pad_main - int_len) {
        output.push_str(" ").unwrap();
    }
    ufmt::uwrite!(output, "{} {}", value, unit).unwrap();

    output
}

// TODO: Works only on positive values, precision must be <=4
fn format_float_measurement(
    value: f32,
    pad_main: u8,
    precision: u8,
    unit: &str,
) -> heapless::String<16> {
    let mut output: heapless::String<16> = heapless::String::new();
    let int_part = value.floor() as u32;

    let int_len = u32_len(int_part);
    for _ in 0..(pad_main - int_len) {
        output.push_str(" ").unwrap();
    }
    let mut frac_text: heapless::String<5> = heapless::String::new();
    if precision > 0 {
        frac_text.push_str(".").unwrap();
        let times = 10_u32.pow(precision as u32);
        let frac_part = (value.fract() * times as f32) as u32;
        let frac_len = u32_len(frac_part);
        for _ in 0..(precision - frac_len) {
            frac_text.push_str("0").unwrap();
        }
        if frac_part > 0 {
            ufmt::uwrite!(frac_text, "{}", frac_part).unwrap();
        }
    }
    ufmt::uwrite!(output, "{}{} {}", int_part, frac_text.as_str(), unit).unwrap();

    output
}

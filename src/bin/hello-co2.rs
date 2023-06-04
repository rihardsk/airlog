#![no_main]
#![no_std]

use cortex_m::prelude::{_embedded_hal_blocking_delay_DelayMs, _embedded_hal_timer_CountDown};
use embedded_hal::blocking::i2c;
use hal::{
    gpio::Level,
    pac::SPI1,
    prelude::OutputPin,
    pwm::{self, Pwm},
    twim, Temp, Twim,
};
use hd44780_driver::{Cursor, CursorBlink, Direction, Display, DisplayMode, HD44780};
use micromath::F32Ext;
use nrf52840_hal::{self as hal, gpio::p0::Parts as P0Parts, gpio::p1::Parts as P1Parts, Timer};

use airlog::{
    self as _,
    logic::{
        self,
        formatting::{format_float_measurement, format_u32_measurement},
    },
    peripherals::{
        button::Button,
        led::{LEDControl, PwmLEDControl},
        scd30::{SensorReading, SCD30},
        sgp40::SGP40,
    },
};
use smart_leds::{SmartLedsWrite, RGB8};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    let board = hal::pac::Peripherals::take().unwrap();
    let core_peripherals = hal::pac::CorePeripherals::take().unwrap();
    let pins_0 = P0Parts::new(board.P0);
    let pins_1 = P1Parts::new(board.P1);
    let mut temp = Temp::new(board.TEMP);
    let mut button = Button::new(pins_0.p0_11.into_pullup_input());

    let mut builtin_led_1 = pins_0.p0_13.into_push_pull_output(Level::High);

    let mut periodic_timer = Timer::periodic(board.TIMER0);
    // let lcd_timer = DurationTimer(Timer::one_shot(board.TIMER1));
    // let mut lcd_timer = Timer::one_shot(board.TIMER1);
    let mut lcd_timer = hal::Delay::new(core_peripherals.SYST);
    let mut sgp40_timer = Timer::one_shot(board.TIMER1);
    let mut sps30_timer = Timer::one_shot(board.TIMER2);

    defmt::info!("Setting up neopixels");
    let pin_smartled = pins_1.p1_08.into_push_pull_output(Level::Low).degrade();
    let mut smartled = nrf_smartled::pwm::Pwm::new(board.PWM0, pin_smartled);
    smartled
        .write(
            [
                RGB8::new(15, 0, 0),
                RGB8::default(),
                RGB8::default(),
                RGB8::default(),
            ]
                .into_iter(),
        )
        .unwrap();
    periodic_timer.delay_ms(300_u32);
    smartled
        .write(
            [
                RGB8::default(),
                RGB8::new(15, 0, 0),
                RGB8::default(),
                RGB8::default(),
            ]
                .into_iter(),
        )
        .unwrap();
    periodic_timer.delay_ms(300_u32);
    smartled
        .write(
            [
                RGB8::default(),
                RGB8::default(),
                RGB8::new(15, 0, 0),
                RGB8::default(),
            ]
                .into_iter(),
        )
        .unwrap();
    periodic_timer.delay_ms(300_u32);
    smartled
        .write(
            [
                RGB8::default(),
                RGB8::default(),
                RGB8::default(),
                RGB8::new(15, 0, 0),
            ]
            .into_iter(),
        )
        .unwrap();
    periodic_timer.delay_ms(300_u32);

    // TODO: mby shine some pretty colors with the smartleds
    // led.set_color(255, 0, 0);
    // periodic_timer.delay_ms(300_u32);
    // led.set_color(0, 255, 0);
    // periodic_timer.delay_ms(300_u32);
    // led.set_color(0, 0, 255);
    // periodic_timer.delay_ms(300_u32);
    // led.set_color(0, 0, 0);
    // periodic_timer.delay_ms(300_u32);

    smartled
        .write(
            [
                RGB8::new(0, 15, 15),
                RGB8::default(),
                RGB8::default(),
                RGB8::default(),
            ]
            .into_iter(),
        )
        .unwrap();
    periodic_timer.delay_ms(300_u32);

    let scl = pins_1.p1_04.into_floating_input().degrade();
    let sda = pins_1.p1_05.into_floating_input().degrade();
    let twim_pins = twim::Pins { scl, sda };
    let i2c = Twim::new(board.TWIM0, twim_pins, twim::Frequency::K100);
    // As long as we only use a single task/thread, we can use BusManagerSimple
    let i2c_bus = shared_bus::BusManagerSimple::new(i2c);
    let i2c_proxy_scd30 = i2c_bus.acquire_i2c();
    let i2c_proxy_sgp40 = i2c_bus.acquire_i2c();
    let i2c_proxy_bmp388 = i2c_bus.acquire_i2c();
    let i2c_proxy_sps30 = i2c_bus.acquire_i2c();

    defmt::info!("Setting up SCD30");

    let mut scd30 = SCD30::new(i2c_proxy_scd30);
    smartled
        .write(
            [
                RGB8::new(15, 0, 0),
                RGB8::default(),
                RGB8::default(),
                RGB8::default(),
            ]
            .into_iter(),
        )
        .unwrap();
    let version = scd30.get_firmware_version().unwrap();
    smartled
        .write(
            [
                RGB8::new(0, 15, 0),
                RGB8::default(),
                RGB8::default(),
                RGB8::default(),
            ]
            .into_iter(),
        )
        .unwrap();
    defmt::info!(
        "SCD30 firmware version: {=u8}.{=u8}",
        version.major,
        version.minor
    );
    smartled
        .write(
            [
                RGB8::new(15, 15, 0),
                RGB8::default(),
                RGB8::default(),
                RGB8::default(),
            ]
            .into_iter(),
        )
        .unwrap();
    let desired_offset: f32 = 3.72;
    let temperature_offset = scd30.read_temperature_offset().unwrap();
    defmt::info!(
        "SCD30 – current temp. offset: {=f32}, desired offset: {=f32}",
        temperature_offset,
        desired_offset
    );
    if temperature_offset != desired_offset {
        defmt::info!("SCD30 – setting temp. offset to {=f32}", desired_offset);
        scd30.set_temperature_offset(desired_offset).unwrap();
    }
    smartled
        .write(
            [
                RGB8::new(0, 0, 15),
                RGB8::default(),
                RGB8::default(),
                RGB8::default(),
            ]
            .into_iter(),
        )
        .unwrap();

    // Just shine some pretty colors in a loop for a while
    let mut color_history = [RGB8::default(); 40];
    for i in 0..=100 {
        let fraction = (i % 100) as f32 / 100.;
        let (r, g, b) = logic::colormap::co2_map_rgb(fraction);
        // Scale the values so that we retain eyesight
        let r = (r as f32 / 8.) as u8;
        let g = (g as f32 / 8.) as u8;
        let b = (b as f32 / 8.) as u8;
        let rgb = RGB8::new(r, g, b);
        let l = color_history.len();
        color_history[i % l] = rgb;
        let past_color_1 = if i >= 9 {
            color_history[(l + i - 9) % l]
        } else {
            rgb
        };
        let past_color_2 = if i >= 19 {
            color_history[(l + i - 19) % l]
        } else {
            past_color_1
        };
        let past_color_3 = if i >= 29 {
            color_history[(l + i - 29) % l]
        } else {
            past_color_2
        };

        smartled
            .write([rgb, past_color_1, past_color_2, past_color_3].into_iter())
            .unwrap();

        periodic_timer.delay_ms(30_u32);
    }
    periodic_timer.delay_ms(100_u32);

    defmt::info!("Initializing SGP40 VOC sensor");
    // NOTE: don't forget that there must be atleast 0.6ms of delay before
    // making the first measurement
    let mut sgp40 = SGP40::new(i2c_proxy_sgp40, 1.);

    defmt::info!("Initializing BMP388 pressure sensor");
    // 0x76 is the address we get when the SDO pin of BMP388 is connected to
    // ground (as per documentation)
    let mut bmp388 = bmp388::BMP388::new(i2c_proxy_bmp388, 0x76, &mut periodic_timer).ok();
    // The choices here are rather arbitrary
    bmp388.as_mut().map(|sensor| {
        sensor
            .set_power_control(bmp388::PowerControl {
                pressure_enable: true,
                temperature_enable: true,
                mode: bmp388::PowerMode::Normal,
            })
            .unwrap()
    });
    bmp388.as_mut().map(|sensor| {
        sensor
            .set_sampling_rate(bmp388::SamplingRate::ms10)
            .unwrap()
    });
    bmp388
        .as_mut()
        .map(|sensor| sensor.set_filter(bmp388::Filter::c3).unwrap());
    bmp388.as_mut().map(|sensor| {
        sensor
            .set_oversampling(bmp388::OversamplingConfig {
                osr_p: bmp388::Oversampling::x4,
                osr4_t: bmp388::Oversampling::x1,
            })
            .unwrap()
    });

    defmt::info!("Initializing SPS30 particulate matter sensor");
    // TODO: do we need to pull the i2c lines up to 5V (in hardware, as per the
    // datasheet)? Seems to be doing ok without it, though
    let mut sps30 = sps30_i2c::Sps30::new_sps30(i2c_proxy_sps30, sps30_timer);
    let mut random = hal::rng::Rng::new(board.RNG);
    let rand_u8 = random.random_u8();
    // Perform cleaning with p = 0.1, so that we don't needlessly do it on every
    // startup. There's also automatic cleaning, which we don't have to manage.
    // We do manual cleaning for occasions where the sensor doesn't run long
    // enough to trigger automatic cleaning (which happens after a week of
    // runtime).
    let threshold = (u8::MAX as f32 / 10.) as u8;
    defmt::info!(
        "Rolled {=u8} (must not exceed {=u8} to perform cleaning)",
        rand_u8,
        threshold
    );
    if rand_u8 < threshold {
        defmt::info!("Performing SPS30 cleaning");
        sps30.start_fan_cleaning();
    } else {
        defmt::info!("NOT performing SPS30 cleaning");
    }
    sps30.start_measurement();

    defmt::info!("Initializing LCD");
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
        rel_humidity: 50.,
        temperature: 25.,
    };
    let mut voc_index: u16;
    let mut builtin_led_state = hal::prelude::PinState::Low;
    let mut builtin_temperature: f32 = 25.;
    let mut pressure_data: Option<bmp388::SensorData>;
    let mut rgb_co2 = RGB8::default();
    let mut rgb_voc = RGB8::default();
    let mut rgb_temp = RGB8::default();
    let mut rgb_pm10 = RGB8::default();
    let mut pm25_data = sps30_i2c::AirInfo::default();
    let mut lcd_output_type = InfoType::GasesAndParticles;
    periodic_timer.start(1_000_000_u32);
    loop {
        // One loop iteration runs for a second, so the button must be held for
        // at least that long for this line to trigger
        if button.check_rising_edge() {
            lcd_output_type = lcd_output_type.next();
            defmt::info!("Switched output to {:?}", lcd_output_type);
        }

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
            let (r, g, b) = logic::colormap::co2_map_rgb(fraction);
            // Scale the values so that we retain eyesight
            let r = (r as f32 / 8.) as u8;
            let g = (g as f32 / 8.) as u8;
            let b = (b as f32 / 8.) as u8;
            rgb_co2 = RGB8::new(r, g, b);
            smartled
                .write([rgb_co2, rgb_voc, rgb_temp, rgb_pm10].into_iter())
                .unwrap();
        }
        if seconds % 3 == 0 {
            loop {
                if sps30.read_data_ready_flag().unwrap() {
                    break;
                }
            }
            pm25_data = sps30.read_measured_values().unwrap();

            let fraction = pm25_data.mass_pm10 / 50.;
            let fraction = fraction.max(0.);
            let (r, g, b) = logic::colormap::pm10_map_rgb(fraction);
            // Scale the values so that we retain eyesight
            let r = (r as f32 / 8.) as u8;
            let g = (g as f32 / 8.) as u8;
            let b = (b as f32 / 8.) as u8;
            rgb_pm10 = RGB8::new(r, g, b);
            smartled
                .write([rgb_co2, rgb_voc, rgb_temp, rgb_pm10].into_iter())
                .unwrap();
        }

        if seconds % 5 == 0 {
            builtin_temperature = temp.measure().to_num();

            let fraction = builtin_temperature / 45.;
            let fraction = fraction.max(0.);
            let (r, g, b) = logic::colormap::temp_map_rgb(fraction);
            // Scale the values so that we retain eyesight
            let r = (r as f32 / 8.) as u8;
            let g = (g as f32 / 8.) as u8;
            let b = (b as f32 / 8.) as u8;
            rgb_temp = RGB8::new(r, g, b);
            smartled
                .write([rgb_co2, rgb_voc, rgb_temp, rgb_pm10].into_iter())
                .unwrap();
        }

        let voc_temp = builtin_temperature.round() as i16;
        let voc_humidity = reading.rel_humidity.round() as u8;
        voc_index = sgp40
            .measure_signal_compensated(voc_temp, voc_humidity, &mut sgp40_timer)
            .unwrap();
        let fraction = voc_index as f32 / 500.;
        let fraction = fraction.max(0.);
        let (r, g, b) = logic::colormap::voc_map_rgb(fraction);
        // Scale the values so that we retain eyesight
        let r = (r as f32 / 8.) as u8;
        let g = (g as f32 / 8.) as u8;
        let b = (b as f32 / 8.) as u8;
        rgb_voc = RGB8::new(r, g, b);
        smartled
            .write([rgb_co2, rgb_voc, rgb_temp, rgb_pm10].into_iter())
            .unwrap();

        if seconds % 5 == 0 {
            pressure_data = bmp388
                .as_mut()
                .and_then(|sensor| sensor.sensor_values().ok());

            // TODO: figure out how to specify {=Option<f64>} explicitly
            defmt::info!(
                "
                CO2: {=f32} ppm
                Temperature: {=f32} °C
                Temp. builtin: {=f32} °C
                Temp. bmp388: {} °C
                Temp. diff: {=f32} °C
                Rel. humidity: {=f32} %
                VOC idx: {=u16}
                Pressue: {} Pa
                ====== Particles ======
                Mass concentration PM1.0: {=f32} μg/m³
                Mass concentration PM2.5: {=f32} μg/m³
                Mass concentration PM4.0: {=f32} μg/m³
                Mass concentration PM10: {=f32} μg/m³
                Number concentration PM0.5: {=f32} #/cm³
                Number concentration PM1.0: {=f32} #/cm³
                Number concentration PM2.5: {=f32} #/cm³
                Number concentration PM4.0: {=f32} #/cm³
                Number concentration PM10: {=f32} #/cm³
                Typical size: {=f32} μm
            ",
                reading.co2,
                reading.temperature,
                builtin_temperature,
                pressure_data.map(|rust_is_too_verbose| rust_is_too_verbose.temperature),
                reading.temperature - builtin_temperature,
                reading.rel_humidity,
                voc_index,
                pressure_data.map(|rust_is_too_verbose| rust_is_too_verbose.pressure),
                pm25_data.mass_pm1_0,
                pm25_data.mass_pm2_5,
                pm25_data.mass_pm4_0,
                pm25_data.mass_pm10,
                pm25_data.number_pm0_5,
                pm25_data.number_pm1_0,
                pm25_data.number_pm2_5,
                pm25_data.number_pm4_0,
                pm25_data.number_pm10,
                pm25_data.typical_size,
            );

            lcd.set_cursor_pos(0, &mut lcd_timer).unwrap();
            let co2_text = format_float_measurement(reading.co2, 4, 0, "ppm");
            lcd.write_str(&co2_text, &mut lcd_timer).unwrap();

            lcd.shift_cursor(Direction::Right, &mut lcd_timer).unwrap();
            // TODO: can we make u32 stuff generic?
            let voc_text = format_u32_measurement(voc_index as u32, 3, "voc");
            lcd.write_str(&voc_text, &mut lcd_timer).unwrap();

            match lcd_output_type {
                InfoType::GasesAndTemp => {
                    lcd.set_cursor_pos(40, &mut lcd_timer).unwrap();
                    // TODO: Can't output °, because it's probably part of unicode, not
                    // ascii, See if there's a workaround using the hd44780 font table
                    let temp_text = format_float_measurement(builtin_temperature, 2, 2, "C");
                    lcd.write_str(&temp_text, &mut lcd_timer).unwrap();

                    lcd.shift_cursor(Direction::Right, &mut lcd_timer).unwrap();
                    lcd.shift_cursor(Direction::Right, &mut lcd_timer).unwrap();
                    let humidity_text = format_float_measurement(reading.rel_humidity, 2, 2, "%");
                    lcd.write_str(&humidity_text, &mut lcd_timer).unwrap();
                }
                InfoType::GasesAndParticles => {
                    lcd.set_cursor_pos(40, &mut lcd_timer).unwrap();
                    let pm25_text = format_float_measurement(pm25_data.mass_pm2_5, 2, 1, "ug");
                    lcd.write_str(&pm25_text, &mut lcd_timer).unwrap();

                    lcd.shift_cursor(Direction::Right, &mut lcd_timer).unwrap();
                    lcd.shift_cursor(Direction::Right, &mut lcd_timer).unwrap();
                    let pm10_text = format_float_measurement(pm25_data.mass_pm10, 2, 1, "ug");
                    lcd.write_str(&pm10_text, &mut lcd_timer).unwrap();
                }
            }
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

#[derive(Debug, Copy, Clone, defmt::Format)]
enum InfoType {
    /// Display CO2, VOC index, temperature and relative humidity
    GasesAndTemp,
    /// Display CO2, VOC index, and PM2.5 and PM10 mass concentrations
    GasesAndParticles,
}

impl InfoType {
    pub fn next(&self) -> Self {
        match self {
            Self::GasesAndTemp => Self::GasesAndParticles,
            Self::GasesAndParticles => Self::GasesAndTemp,
        }
    }
}

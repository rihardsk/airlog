#![no_main]
#![no_std]

use cortex_m::prelude::_embedded_hal_timer_CountDown;
use hal::Temp;
use nrf52840_hal::{self as hal, gpio::p0::Parts as P0Parts, Timer};

use airlog::{self as _, peripherals::button::Button}; // global logger + panicking-behavior + memory layout

#[derive(defmt::Format)]
enum TempUnit {
    Kelvin,
    Celsius,
    Fahrenheit,
}

impl TempUnit {
    fn convert_from_celsius(&self, temp_in_celsius: f32) -> f32 {
        match self {
            TempUnit::Kelvin => temp_in_celsius - 273.15,
            TempUnit::Celsius => temp_in_celsius,
            TempUnit::Fahrenheit => temp_in_celsius * 9f32 / 5f32 + 32f32,
        }
    }

    fn get_symbol(&self) -> &'static str {
        match self {
            TempUnit::Kelvin => "K",
            TempUnit::Celsius => "°C",
            TempUnit::Fahrenheit => "°F",
        }
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("Hello, world!");

    let board = hal::pac::Peripherals::take().unwrap();
    let pins = P0Parts::new(board.P0);
    let mut temp = Temp::new(board.TEMP);

    let mut current_unit = TempUnit::Celsius;
    let mut button = Button::new(pins.p0_11.into_floating_input());
    let mut periodic_timer = Timer::periodic(board.TIMER0);
    let mut millis: u32 = 0;

    periodic_timer.start(1000_u32);
    loop {
        if (millis % 1000) == 0 {
            let temperature: f32 = temp.measure().to_num();
            let converted_temp = current_unit.convert_from_celsius(temperature);
            let converted_symbol = current_unit.get_symbol();
            defmt::info!("{=f32} {}", converted_temp, converted_symbol);
        }

        if (millis % 5) == 0 && button.check_rising_edge() {
            current_unit = match current_unit {
                TempUnit::Fahrenheit => TempUnit::Kelvin,
                TempUnit::Kelvin => TempUnit::Celsius,
                TempUnit::Celsius => TempUnit::Fahrenheit,
            };
            defmt::info!("Unit changed to {}", current_unit);
        }

        nb::block!(periodic_timer.wait()).unwrap();
        millis = millis.overflowing_add(1).0;
    }
}

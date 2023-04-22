#![no_main]
#![no_std]

pub mod logic;
pub mod peripherals;

use defmt_rtt as _; // global logger

// TODO(5) adjust HAL import
// use some_hal as _; // memory layout
use nrf52840_hal as _; // memory layout

use panic_probe as _;

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

// defmt-test 0.3.0 has the limitation that this `#[tests]` attribute can only be used
// once within a crate. the module can be in any file but there can only be at most
// one `#[tests]` module in this library crate
#[cfg(test)]
#[defmt_test::tests]
mod unit_tests {
    use defmt::assert;
    use super::peripherals::sgp40::tests as sgp40_tests;
    use super::logic::formatting::tests as formatting_tests;

    #[test]
    fn it_works() {
        assert!(true)
    }

    #[test]
    fn generate_command() {
        sgp40_tests::generate_command();
    }

    #[test]
    fn format_zero() {
        formatting_tests::format_zero();
    }

    #[test]
    fn format_ten() {
        formatting_tests::format_ten();
    }

    #[test]
    fn format_single_digit() {
        formatting_tests::format_single_digit();
    }

    #[test]
    fn format_all_digits() {
        formatting_tests::format_all_digits();
    }

    #[test]
    fn format_more_digits() {
        formatting_tests::format_more_digits();
    }

    #[test]
    fn format_dont_pad() {
        formatting_tests::format_dont_pad();
    }

    #[test]
    fn format_float_zero() {
        formatting_tests::format_float_zero();
    }

    #[test]
    fn format_float_small_fract() {
        formatting_tests::format_float_small_fract();
    }

    #[test]
    fn format_float_smaller_fract() {
        formatting_tests::format_float_smaller_fract();
    }

    #[test]
    fn format_float_single_digit() {
        formatting_tests::format_float_single_digit();
    }

    #[test]
    fn format_float_more_digits() {
        formatting_tests::format_float_more_digits();
    }

    #[test]
    fn format_float_carry_over() {
        formatting_tests::format_float_carry_over();
    }
}

use hal::{
    gpio::{Input, Level, Output, Pin, PullUp, PushPull},
    prelude::{InputPin, OutputPin},
};
use nrf52840_hal as hal;

pub struct Button {
    pin: Pin<Input<PullUp>>,
    was_pressed: bool,
}

impl Button {
    pub fn new<Mode>(pin: Pin<Mode>) -> Self {
        Button {
            pin: pin.into_pullup_input(),
            was_pressed: false,
        }
    }

    pub fn is_pressed(&self) -> bool {
        self.pin.is_low().unwrap()
    }

    /// Check when button changes state from being pressed to not being pressed
    pub fn check_rising_edge(&mut self) -> bool {
        let is_pressed = self.is_pressed();
        let rising_edge = self.was_pressed && !is_pressed;
        self.was_pressed = is_pressed;
        rising_edge
    }
}

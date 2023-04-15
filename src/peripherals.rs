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

pub struct LEDControl {
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

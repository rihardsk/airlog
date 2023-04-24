use embedded_hal::digital::v2::InputPin;

pub struct Button<P> {
    pin: P,
    was_pressed: bool,
}

impl<P> Button<P>
where
    P: InputPin,
    // This is somehow needed for .unwrap()
    P::Error: core::fmt::Debug,
{
    // TODO: check that callers do an into_pullup?
    // what is pullup really?
    pub fn new(pin: P) -> Self {
        Button {
            pin,
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

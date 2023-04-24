use crate::future::pwm::SetDutyCycle;
use embedded_hal::digital::v2::OutputPin;

pub struct PwmLEDControl<T> {
    channel_red: T,
    channel_green: T,
    channel_blue: T,
}

impl<T> PwmLEDControl<T>
where
    T: SetDutyCycle,
{
    // TODO: take individual channels
    pub fn new(red: T, green: T, blue: T) -> Self {
        PwmLEDControl {
            channel_red: red,
            channel_blue: blue,
            channel_green: green,
        }
    }

    pub fn set_color(&mut self, red: u8, green: u8, blue: u8) {
        self.channel_red
            .set_duty_cycle_fraction(red as u16, 255_u16)
            .unwrap();
        self.channel_green
            .set_duty_cycle_fraction(green as u16, 255_u16)
            .unwrap();
        self.channel_blue
            .set_duty_cycle_fraction(blue as u16, 255_u16)
            .unwrap();
    }

    pub fn free(self) -> (T, T, T) {
        (self.channel_red, self.channel_green, self.channel_blue)
    }
}

pub struct LEDControl<T> {
    r: T,
    g: T,
    b: T,
}

impl<T> LEDControl<T>
where
    T: OutputPin,
    T::Error: core::fmt::Debug,
{
    pub fn new(led_red: T, led_green: T, led_blue: T) -> Self {
        let mut led = LEDControl {
            r: led_red,
            g: led_green,
            b: led_blue,
        };
        // TODO should probably pass configuration whether the leds are active
        // high or active low
        led.r.set_high().unwrap();
        led.g.set_high().unwrap();
        led.b.set_high().unwrap();
        led
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

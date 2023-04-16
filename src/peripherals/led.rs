use hal::{
    gpio::{Input, Level, Output, Pin, PullUp, PushPull},
    prelude::{InputPin, OutputPin},
    pwm::{Instance, Pwm, Channel},
};
use nrf52840_hal as hal;

pub struct PwmLEDControl<T: Instance> {
    pwm: Pwm<T>,
}

impl<T> PwmLEDControl<T>
where
    T: Instance,
{
    // TODO: take individual channels
    pub fn new(pwm: Pwm<T>) -> Self {
        pwm.set_max_duty(255);
        PwmLEDControl { pwm }
    }

    pub fn set_color(&mut self, red: u8, green: u8, blue: u8) {
        self.pwm.set_duty_on(Channel::C0, red as u16);
        self.pwm.set_duty_on(Channel::C1, green as u16);
        self.pwm.set_duty_on(Channel::C2, blue as u16);
    }

    pub fn free(self) -> Pwm<T> {
        self.pwm
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

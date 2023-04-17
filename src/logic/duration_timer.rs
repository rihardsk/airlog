use core::time::Duration;

use nrf52840_hal::{Timer, timer::OneShot, timer::Instance};
use void::Void;

pub struct DurationTimer<T>(pub Timer<T, OneShot>);

impl<I> embedded_hal::timer::CountDown for DurationTimer<I> where I: Instance {
    type Time = Duration;

    fn start<T>(&mut self, count: T)
    where
        T: Into<Self::Time> {
        let duration: Duration = count.into();
        // I guess it's better to crash if we can't wait as long as needed, than
        // wait a wrong amount of time. We'll be using this type only for short
        // durations, though (timing stuff when sending stuff over the wire)
        let micros = u32::try_from(duration.as_micros()).unwrap();
        // Hopefuly `start()` takes microseconds. Checking the implementation of
        // `delay_us()`, it seems it does, because it calls `start()` underneath
        // just like this.
        self.0.start(micros);
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        self.0.wait()
    }
}

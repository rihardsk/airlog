use crc_all::Crc;
use embedded_hal::blocking::{delay::DelayMs, i2c};
use gas_index_algorithm::{AlgorithmType, GasIndexAlgorithm};

// pub struct FirmwareVersion {
//     pub major: u8,
//     pub minor: u8,
// }

pub struct SGP40<T> {
    i2c: T,
    algo: GasIndexAlgorithm,
}

static DEFAULT_ADDRESS: u8 = 0x59;
static DEFAULT_MEASUREMENT_COMMAND: [u8; 8] = [0x26, 0x0f, 0x80, 0x00, 0xa2, 0x66, 0x66, 0x93];

pub struct SensorReading {
    pub voc_index: f32,
    pub voc_raw: f32,
}

impl<T> SGP40<T>
where
    T: i2c::Write,
    T: i2c::Read<Error = <T as i2c::Write>::Error>,
{
    /// NOTE: The sensor is ready to receive commands from the i2c master 0.6ms
    /// after powering on (after reaching voltage of 1.7V).
    pub fn new(i2c: T, sampling_interval_s: f32) -> Self {
        let algo = GasIndexAlgorithm::new(AlgorithmType::Voc, sampling_interval_s);
        SGP40 { i2c, algo }
    }

    // TODO: there's a serial number not a firmware version available only
    // pub fn get_firmware_version(&mut self) -> Result<FirmwareVersion, Error> {
    // }

    pub fn measure_raw_signal_compensated(
        &mut self,
        temperature: i16,
        humidity: u8,
        delay: &mut impl DelayMs<u8>,
    ) -> Result<u16, <T as i2c::Write>::Error> {
        let command = create_measurement_command(temperature, humidity);
        self.i2c.write(DEFAULT_ADDRESS, &command)?;
        delay.delay_ms(30);
        let mut buf = [0; 3];
        self.i2c.read(DEFAULT_ADDRESS, &mut buf)?;
        let sraw_voc = u16::from_be_bytes([buf[0], buf[1]]);
        Ok(sraw_voc)
    }

    pub fn measure_signal_compensated(
        &mut self,
        temperature: i16,
        humidity: u8,
        delay: &mut impl DelayMs<u8>,
    ) -> Result<u16, <T as i2c::Write>::Error> {
        let sraw_voc = self.measure_raw_signal_compensated(temperature, humidity, delay)?;
        let voc_idx = self.algo.process(sraw_voc as i32);
        Ok(voc_idx as u16)
    }
}

fn create_measurement_command(temperature: i16, humidity: u8) -> [u8; 8] {
    let mut command: [u8; 8] = [0x26, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let humidity_bytes = humidity_to_ticks(humidity).to_be_bytes();
    command[2] = humidity_bytes[0];
    command[3] = humidity_bytes[1];

    let mut crc = Crc::<u8>::new(0x31, 8, 0xff, 0x00, false);
    crc.update(&humidity_bytes);
    command[4] = crc.finish();

    let temp_bytes = temperature_to_ticks(temperature).to_be_bytes();
    command[5] = temp_bytes[0];
    command[6] = temp_bytes[1];

    let mut crc = Crc::<u8>::new(0x31, 8, 0xff, 0x00, false);
    crc.update(&temp_bytes);
    command[7] = crc.finish();

    command
}

/// Value must be in range [-45; 130]
fn temperature_to_ticks(value: i16) -> u16 {
    let (quot, rem) = div_rem((value + 45) as u32 * 65535, 175);
    // Use <= here, because 175 / 2 get's rounded down
    if rem <= 175 / 2 {
        quot as u16
    } else {
        (quot + 1) as u16
    }
}

/// Value must be in range [0; 100]
fn humidity_to_ticks(value: u8) -> u16 {
    let (quot, rem) = div_rem(value as u32 * 65535, 100);
    if rem < 100 / 2 {
        quot as u16
    } else {
        (quot + 1) as u16
    }
}

// There's hope that rustc can optimize this into a sinlge instruction, at least on some plaforms
fn div_rem(x: u32, y: u32) -> (u32, u32) {
    let quot = x / y;
    let rem = x % y;
    (quot, rem)
}

#[cfg(test)]
pub mod tests {
    use super::{create_measurement_command, DEFAULT_MEASUREMENT_COMMAND};

    // Defmt isn't too good at tests curently
    // #[test]
    pub fn generate_command() {
        let generated_command = create_measurement_command(25, 50);
        assert_eq!(generated_command, DEFAULT_MEASUREMENT_COMMAND);
    }
}

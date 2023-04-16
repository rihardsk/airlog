use crc_all::Crc;
use nrf52840_hal::{twim::{Instance, Error}, Twim};

pub struct SCD30<T: Instance>(Twim<T>);

static DEFAULT_ADDRESS: u8 = 0x61;

pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8,
}

impl<T> SCD30<T>
where
    T: Instance,
{
    // TODO: scd30 interface description mentions something about i2c clock
    // stretching and that we need to support it. find out what that is
    pub fn new(i2c2: Twim<T>) -> Self {
        SCD30(i2c2)
    }

    pub fn get_firmware_version(&mut self) -> Result<FirmwareVersion, Error> {
        let command: [u8; 2] = [0xd1, 0x00];
        // Interesting, if we inline the command array, we get a
        // DMABufferNotInDataMemory error. Seems like the command must be in
        // data region of the memory for this to work.
        self.0.write(DEFAULT_ADDRESS, &command)?;
        let mut buf = [0; 3];
        self.0.read(DEFAULT_ADDRESS, &mut buf)?;

        let major = u8::from_be(buf[0]);
        let minor = u8::from_be(buf[1]);
        // TODO: do something with the checksum

        Ok(FirmwareVersion { major, minor })
    }

    pub fn start_continuous_measurement(&mut self, pressure: u16) -> Result<(), Error> {
        let mut command: [u8; 5] = [0x00, 0x10, 0x00, 0x00, 0x00];
        let pressure_bytes = pressure.to_be_bytes();
        command[2] = pressure_bytes[0];
        command[3] = pressure_bytes[1];

        let mut crc = Crc::<u8>::new(0x31, 8, 0xff, 0x00, false);
        crc.update(&pressure_bytes);
        command[4] = crc.finish();

        self.0.write(DEFAULT_ADDRESS, &command)?;

        Ok(())
    }

    pub fn data_ready(&mut self) -> Result<bool, Error> {
        let command: [u8; 2] = [0x02, 0x02];
        self.0.write(DEFAULT_ADDRESS, &command)?;
        let mut buf = [0; 3];
        self.0.read(DEFAULT_ADDRESS, &mut buf)?;

        // TODO: check crc
        Ok(u16::from_be_bytes([buf[0], buf[1]]) == 1)
    }
}

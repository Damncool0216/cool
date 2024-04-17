#![no_std]

mod conf;
mod register;

pub use accelerometer;
pub use conf::{DataFormatFlags, DataFormatRange};
use embedded_hal as hal;

#[cfg(feature = "u16x3")]
use accelerometer::vector::U16x3;
#[cfg(feature = "i16x3")]
use accelerometer::{
    vector::{F32x3, I16x3},
    Accelerometer,
};
use register::Register;

use accelerometer::{Error, ErrorKind, RawAccelerometer};
use core::fmt::Debug;

use hal::spi::SpiBus;

pub const DEVICE_ID: u8 = 0xE5;

const SPI_READ: u8 = 0x80;
const SPI_WRITE: u8 = 0x00;

pub struct Adxl345<SPI> {
    /// spi driver
    spi: SPI,
    /// Current data format
    data_format: DataFormatFlags,
}

impl<SPI, E> Adxl345<SPI>
where
    SPI: SpiBus<Error = E>,
    E: Debug,
{
    pub fn default(spi: SPI) -> Result<Self, Error<E>> {
        Self::new_with_data_format(spi, DataFormatFlags::default())
    }
    /// Create a new ADXL345 driver configured with the given data format
    pub fn new_with_data_format<F>(spi: SPI, data_format: F) -> Result<Self, Error<E>>
    where
        F: Into<DataFormatFlags>,
    {
        let mut adxl = Adxl345 {
            spi,
            data_format: DataFormatFlags::default(),
        };

        // Ensure we have the correct device ID for the ADLX345
        if adxl.get_device_id()? != DEVICE_ID {
            ErrorKind::Device.err()?;
        }

        // Configure the data format
        adxl.set_data_format(data_format)?;

        // Disable interrupts
        adxl.write_reg(Register::INT_ENABLE, 0)?;

        // 62.5 mg/LSB
        adxl.write_reg(Register::THRESH_TAP, 20)?;

        // Tap duration: 625 µs/LSB
        adxl.write_reg(Register::DUR, 50)?;

        // Tap latency: 1.25 ms/LSB (0 = no double tap)
        adxl.write_reg(Register::LATENT, 0)?;

        // Waiting period: 1.25 ms/LSB (0 = no double tap)
        adxl.write_reg(Register::WINDOW, 0)?;

        // Enable XYZ axis for tap
        adxl.write_reg(Register::TAP_AXES, 0x7)?;

        // Enable measurements
        adxl.write_reg(Register::POWER_CTL, 0x08)?;

        Ok(adxl)
    }

    /// Set the device data format
    pub fn set_data_format<F>(&mut self, data_format: F) -> Result<(), Error<E>>
    where
        F: Into<DataFormatFlags>,
    {
        let f = data_format.into();
        let bytes = [Register::DATA_FORMAT.addr(), f.bits()];
        self.spi.write(&bytes);
        self.data_format = f;
        Ok(())
    }

    /// Write to the given register
    fn write_reg(&mut self, register: Register, value: u8) -> Result<(), Error<E>> {
        // Preserve the invariant around self.data_format
        assert_ne!(
            register,
            Register::DATA_FORMAT,
            "set data format with Adxl343::data_format"
        );

        debug_assert!(!register.read_only(), "can't write to read-only register");
        let mut bytes = [register.addr(), value];
        self.spi.write(&mut bytes)?;
        Ok(())
    }

    fn read_reg(&mut self, register: Register) -> Result<u8, Error<E>> {
        let mut bytes = [register.addr() | SPI_READ, 0];
        self.spi.transfer_in_place(&mut bytes)?;
        Ok(bytes[1])
    }

    /// Get the device ID
    fn get_device_id(&mut self) -> Result<u8, Error<E>> {
        let dev_id = self.read_reg(Register::DEVID)?;
        Ok(dev_id)
    }

    /// Write to a given register, then read a `i16` result
    ///
    /// From the ADXL343 data sheet (p.25):
    /// <https://www.analog.com/media/en/technical-documentation/data-sheets/adxl343.pdf>
    ///
    /// "The output data is twos complement, with DATAx0 as the least
    /// significant byte and DATAx1 as the most significant byte"
    #[cfg(feature = "i16x3")]
    fn write_read_i16(&mut self, register: Register) -> Result<i16, E> {
        let mut buffer = [0u8; 3];
        buffer[0] = register.addr();
        self.spi.transfer_in_place(&mut buffer)?;
        Ok(i16::from_be_bytes([buffer[1], buffer[2]]))
    }

    /// Write to a given register, then read a `u16` result
    ///
    /// Used for reading `JUSTIFY`-mode data. From the ADXL343 data sheet (p.25):
    /// <https://www.analog.com/media/en/technical-documentation/data-sheets/adxl343.pdf>
    ///
    /// "A setting of 1 in the justify bit selects left-justified (MSB) mode,
    /// and a setting of 0 selects right-justified mode with sign extension."
    #[cfg(feature = "u16x3")]
    fn write_read_u16(&mut self, register: Register) -> Result<u16, E> {
        let mut buffer = [0u8; 3];
        buffer[0] = register.addr();
        self.spi.transfer_in_place(&mut buffer)?;
        Ok(i16::from_le_bytes([buffer[1], buffer[2]]))
    }
}

use bitflags::bitflags;

bitflags! {
    /// Flags passed as operands to `Register::DATA_FORMAT`
    ///
    /// "The DATA_FORMAT register controls the presentation of data
    /// to Register 0x32 through Register 0x37. All data, except that for
    /// the ±16 g range, must be clipped to avoid rollover."
    pub struct DataFormatFlags: u8 {
        /// "A setting of 1 in the SELF_TEST bit applies a self-test force to
        /// the sensor, causing a shift in the output data. A value of 0 disables
        /// the self-test force."
        const SELF_TEST = 0b10000000;

        /// "A value of 1 in the SPI bit sets the device to 3-wire SPI mode,
        /// and a value of 0 sets the device to 4-wire SPI mode"
        const SPI = 0b01000000;

        /// "A value of 0 in the INT_INVERT bit sets the interrupts to active
        /// high, and a value of 1 sets the interrupts to active low."
        const INT_INVERT = 0b00100000;

        /// "When this bit is set to a value of 1, the device is in full resolution
        /// mode, where the output resolution increases with the g range
        /// set by the range bits to maintain a 4 mg/LSB scale factor. When
        /// the FULL_RES bit is set to 0, the device is in 10-bit mode, and
        /// the range bits determine the maximum g range and scale factor"
        const FULL_RES = 0b00001000;

        /// A setting of 1 in the justify bit selects left-justified (MSB) mode,
        /// and a setting of 0 selects right-justified mode with sign extension.
        const JUSTIFY = 0b00000100;

        /// Range high bit (see `DataFormatRange`)
        const RANGE_HI = 0b00000010;

        /// Range low bit (see `DataFormatRange`)
        const RANGE_LO = 0b00000001;
    }
}

impl DataFormatFlags {
    /// Get the [`DataFormatRange`] from the flags
    pub fn range(self) -> DataFormatRange {
        if self.contains(DataFormatFlags::RANGE_HI) {
            if self.contains(DataFormatFlags::RANGE_LO) {
                DataFormatRange::PLUSMINUS_16G
            } else {
                DataFormatRange::PLUSMINUS_8G
            }
        } else if self.contains(DataFormatFlags::RANGE_LO) {
            DataFormatRange::PLUSMINUS_4G
        } else {
            DataFormatRange::PLUSMINUS_2G
        }
    }
}

/// Default `DATA_FORMAT` settings:
///
/// - `SELF_TEST`: false
/// - `SPI`: false
/// - `INT_INVERT`: false
/// - `FULL_RES`: false
/// - `JUSTIFY`: false
/// - Range: ±2g (i.e. 0)
impl Default for DataFormatFlags {
    fn default() -> Self {
        DataFormatFlags::empty()
    }
}

impl From<DataFormatRange> for DataFormatFlags {
    fn from(range: DataFormatRange) -> DataFormatFlags {
        range.bits()
    }
}

/// g-Range setting flags which can be OR'd with `DataFormatFlags` and passed as
/// operands to `Register::DATA_FORMAT`
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum DataFormatRange {
    /// ±2g
    PLUSMINUS_2G = 0b00,

    /// ±4g
    PLUSMINUS_4G = 0b01,

    /// ±8g
    PLUSMINUS_8G = 0b10,

    /// ±16g
    PLUSMINUS_16G = 0b11,
}

impl DataFormatRange {
    /// Get `DataFormatFlags` representation
    pub fn bits(self) -> DataFormatFlags {
        match self {
            DataFormatRange::PLUSMINUS_2G => DataFormatFlags::empty(),
            DataFormatRange::PLUSMINUS_4G => DataFormatFlags::RANGE_LO,
            DataFormatRange::PLUSMINUS_8G => DataFormatFlags::RANGE_HI,
            DataFormatRange::PLUSMINUS_16G => DataFormatFlags::RANGE_HI | DataFormatFlags::RANGE_LO,
        }
    }
}

impl From<DataFormatRange> for f32 {
    fn from(range: DataFormatRange) -> f32 {
        match range {
            DataFormatRange::PLUSMINUS_2G => 2.0,
            DataFormatRange::PLUSMINUS_4G => 4.0,
            DataFormatRange::PLUSMINUS_8G => 8.0,
            DataFormatRange::PLUSMINUS_16G => 16.0,
        }
    }
}
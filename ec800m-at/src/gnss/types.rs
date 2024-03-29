pub enum NmeaConfig {
    Defalut,
    AllEnable,
    AllDisable,
    Config {
        gga: bool,
        rmc: bool,
        gsv: bool,
        gsa: bool,
        vtg: bool,
        gll: bool,
    },
}

pub enum Outport {
    None,
    UsbNmea,
    UartDebug,
}

#[derive(Clone, Debug)]
#[repr(u8)]
pub enum GnssConfig {
    Gps = 0,
    GpsBeiDou = 1,
    GpsGlonassGalileo = 3,
    GpsGlonass = 4,
    GpsBeiDouGalileo = 5,
    GpsGalileo = 6,
    BeiDou = 7,
}

#[derive(Clone)]
#[repr(u8)]
pub enum DeleteType {
    AllDel = 0,
    NotDel = 1,
    PartDel = 2,
}

pub enum NmeaType {
    GGA,
    RMC,
    GSV,
    GSA,
    VTG,
    GLL,
}
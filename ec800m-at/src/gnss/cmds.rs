use crate::general::{
    resps::{NoResp, OkResp},
    types::OnOff,
};
use atat::{atat_derive::AtatCmd, Error, InternalError};
use heapless::String;

use super::{resps::NmeaResp, types::{DeleteType, GnssConfig, NmeaConfig, NmeaType, Outport}};

/// 2.3.1.1 AT+QGPSCFG="outport" 配置NMEA语句输出端口
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QGPSCFG", OkResp, timeout_ms = 600)]
pub struct QGpsCfgOutPortSet {
    #[at_arg(position = 1)]
    cfg: String<10>,
    #[at_arg(position = 2)]
    out_port: String<15>,
}
impl QGpsCfgOutPortSet {
    pub fn new(outport: Outport) -> Self {
        Self {
            cfg: String::try_from("outport").unwrap(),
            out_port: match outport {
                Outport::None => String::try_from("none").unwrap(),
                Outport::UsbNmea => String::try_from("usbnmea").unwrap(),
                Outport::UartDebug => String::try_from("uartdebug").unwrap(),
            },
        }
    }
}

/// 2.3.1.2 AT+QGPSCFG="nmeasrc" 启用/禁用通过 AT+QGPSGNMEA 获取 NMEA 语句
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QGPSCFG", OkResp, timeout_ms = 600)]
pub struct QGpsCfgNmeasrcSet {
    #[at_arg(position = 1)]
    cfg: String<10>,
    #[at_arg(position = 2)]
    on_off: u8,
}
impl QGpsCfgNmeasrcSet {
    pub fn new(on_off: OnOff) -> Self {
        Self {
            cfg: String::try_from("nmeasrc").unwrap(),
            on_off: on_off as u8,
        }
    }
}

/// 2.3.1.3. AT+QGPSCFG="gpsnmeatype" 配置 NMEA 语句的输出类型
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QGPSCFG", OkResp, timeout_ms = 600)]
pub struct QGpsCfgGpsNmeaTypeSet {
    #[at_arg(position = 1)]
    cfg: String<15>,
    #[at_arg(position = 2)]
    gps_nmea_type: u8,
}

impl QGpsCfgGpsNmeaTypeSet {
    pub fn new(nmea_type: NmeaConfig) -> Self {
        Self {
            cfg: String::try_from("gpsnmeatype").unwrap(),
            gps_nmea_type: match nmea_type {
                NmeaConfig::Defalut => 31,
                NmeaConfig::AllEnable => 63,
                NmeaConfig::AllDisable => 0,
                NmeaConfig::Config {
                    gga,
                    rmc,
                    gsv,
                    gsa,
                    vtg,
                    gll,
                } => {
                    let mut cfg = 0;
                    if gga {
                        cfg = cfg | 1;
                    }
                    if rmc {
                        cfg = cfg | 2;
                    }
                    if gsv {
                        cfg = cfg | 4;
                    }
                    if gsa {
                        cfg = cfg | 8;
                    }
                    if vtg {
                        cfg = cfg | 16;
                    }
                    if gll {
                        cfg = cfg | 32;
                    }
                    cfg
                }
            },
        }
    }
}

/// 2.3.1.4. AT+QGPSCFG="gnssconfig" 配置支持的 GNSS 卫星导航系统
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QGPSCFG", OkResp, timeout_ms = 600)]
pub struct QGpsCfgGnssConfigSet {
    #[at_arg(position = 1)]
    cfg: String<15>,
    #[at_arg(position = 2)]
    gnss_confg: u8,
}

impl QGpsCfgGnssConfigSet {
    pub fn new(cfg: GnssConfig) -> Self {
        Self {
            cfg: String::try_from("gnssconfig").unwrap(),
            gnss_confg: cfg as u8,
        }
    }
    pub fn default() -> Self {
        Self {
            cfg: String::try_from("gnssconfig").unwrap(),
            gnss_confg: GnssConfig::GpsBeiDouGalileo as u8,
        }
    }
}

/// 2.3.1.5. AT+QGPSCFG="autogps" 启用/禁用 GNSS 自启动
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QGPSCFG", OkResp, timeout_ms = 600)]
pub struct QGpsCfgAutoGpsSet {
    #[at_arg(position = 1)]
    cfg: String<10>,
    #[at_arg(position = 2)]
    on_off: u8,
}
impl QGpsCfgAutoGpsSet {
    pub fn new(on_off: OnOff) -> Self {
        Self {
            cfg: String::try_from("autogps").unwrap(),
            on_off: on_off as u8,
        }
    }
}

/// 2.3.1.6. AT+QGPSCFG="apflash" 启用/禁用 AP-Flash 快速热启动功能
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QGPSCFG", OkResp, timeout_ms = 600)]
pub struct QGpsCfgApFlashSet {
    #[at_arg(position = 1)]
    cfg: String<10>,
    #[at_arg(position = 2)]
    on_off: u8,
}
impl QGpsCfgApFlashSet {
    pub fn new(on_off: OnOff) -> Self {
        Self {
            cfg: String::try_from("apflash").unwrap(),
            on_off: on_off as u8
        }
    }
}

/// 2.3.2. AT+QGPSDEL 删除辅助数据
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QGPSDEL", OkResp, timeout_ms = 600)]
pub struct QGpsDelSet {
    delete_type: u8,
}
impl QGpsDelSet {
    pub fn new(del: DeleteType) -> Self {
        Self {
            delete_type: del as u8,
        }
    }
}

/// 2.3.3. AT+QGPS 打开 GNSS
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QGPS", OkResp, timeout_ms = 600)]
pub struct QGpsSet {
    on_off: u8,
}
impl QGpsSet {
    pub fn new(on: OnOff) -> Self {
        match on {
            OnOff::On => Self { on_off: 1 },
            OnOff::Off => Self { on_off: 0 },
        }
    }
}
/// 2.3.4. AT+QGPSEND 关闭 GNSS
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QGPSEND", OkResp, timeout_ms = 600)]
pub struct QGpsEndSet;

/// 2.3.5. AT+QGPSLOC 获取定位信息
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QGPSLOC", NoResp)]
pub struct QGpsLocGet {
    mode: u8,
}
impl QGpsLocGet {
    pub fn default() -> Self {
        Self { mode: 0 }
    }
}

/// 2.3.6. AT+QGPSGNMEA 获取指定的 NMEA 语句
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QGPSGNMEA", NmeaResp, timeout_ms = 5000, parse = NmeaResp::parse)]
pub struct QGpsNmeaGet {
    nmea_type: String<3>,
}
impl QGpsNmeaGet {
    pub fn new(nmea_type: NmeaType) -> Self {
        match nmea_type {
            NmeaType::GGA => Self {
                nmea_type: String::try_from("GGA").unwrap(),
            },
            NmeaType::RMC => Self {
                nmea_type: String::try_from("RMC").unwrap(),
            },
            NmeaType::GSV => Self {
                nmea_type: String::try_from("GSV").unwrap(),
            },
            NmeaType::GSA => Self {
                nmea_type: String::try_from("GSA").unwrap(),
            },
            NmeaType::VTG => Self {
                nmea_type: String::try_from("VTG").unwrap(),
            },
            NmeaType::GLL => Self {
                nmea_type: String::try_from("GLL").unwrap(),
            },
        }
    }
}

/// AT+QAGPS 启用/禁用AGPS
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QAGPS", OkResp, timeout_ms = 600)]
pub struct QAgpsSet {
    on_off: u8,
}
impl QAgpsSet {
    pub fn new(on_off: OnOff) -> Self {
        Self {
            on_off: on_off as u8
        }
    }
}

#[cfg(test)]
mod tests {
    use atat::AtatCmd;

    use crate::gnss::cmds::{NmeaConfig, OnOff, QGpsCfgGpsNmeaTypeSet, QGpsCfgNmeasrcSet};

    use super::{
        DeleteType, GnssConfig, QAgpsSet, QGpsCfgAutoGpsSet, QGpsCfgGnssConfigSet,
        QGpsCfgOutPortSet, QGpsDelSet, QGpsEndSet, QGpsGet, QGpsLocGet, QGpsNmeaGet, QGpsSet,
    };

    #[test]
    fn out_port() {
        let cmd = QGpsCfgOutPortSet::new(super::Outport::None);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"outport\",\"none\"\r\n");
        let cmd = QGpsCfgOutPortSet::new(super::Outport::UartDebug);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"outport\",\"uartdebug\"\r\n");
        let cmd = QGpsCfgOutPortSet::new(super::Outport::UsbNmea);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"outport\",\"usbnmea\"\r\n");
    }

    #[test]
    fn nmea_src() {
        let cmd = QGpsCfgNmeasrcSet::new(OnOff::On);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"nmeasrc\",1\r\n");
        let cmd = QGpsCfgNmeasrcSet::new(OnOff::Off);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"nmeasrc\",0\r\n");
    }

    #[test]
    fn gps_nmea_type() {
        let cmd = QGpsCfgGpsNmeaTypeSet::new(NmeaConfig::AllDisable);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"gpsnmeatype\",0\r\n");
        let cmd = QGpsCfgGpsNmeaTypeSet::new(NmeaConfig::AllEnable);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"gpsnmeatype\",64\r\n");
        let cmd = QGpsCfgGpsNmeaTypeSet::new(NmeaConfig::Defalut);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"gpsnmeatype\",31\r\n");
        let cmd = QGpsCfgGpsNmeaTypeSet::new(NmeaConfig::Config {
            gga: true,
            rmc: true,
            gsv: true,
            gsa: true,
            vtg: false,
            gll: false,
        });
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"gpsnmeatype\",31\r\n");
    }

    #[test]
    fn gnss_config() {
        let cmd = QGpsCfgGnssConfigSet::new(GnssConfig::Gps);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"gnssconfig\",0\r\n");
        let cmd = QGpsCfgGnssConfigSet::new(GnssConfig::GpsBeiDou);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"gnssconfig\",1\r\n");
        let cmd = QGpsCfgGnssConfigSet::new(GnssConfig::GpsGlonassGalileo);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"gnssconfig\",3\r\n");
        let cmd = QGpsCfgGnssConfigSet::new(GnssConfig::GpsGlonass);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"gnssconfig\",4\r\n");
        let cmd = QGpsCfgGnssConfigSet::new(GnssConfig::GpsBeiDouGalileo);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"gnssconfig\",5\r\n");
        let cmd = QGpsCfgGnssConfigSet::new(GnssConfig::GpsGalileo);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"gnssconfig\",6\r\n");
        let cmd = QGpsCfgGnssConfigSet::new(GnssConfig::BeiDou);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"gnssconfig\",7\r\n");
    }

    #[test]
    fn auto_gps() {
        let cmd = QGpsCfgAutoGpsSet::new(OnOff::On);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"autogps\",1\r\n");
        let cmd = QGpsCfgAutoGpsSet::new(OnOff::Off);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"autogps\",0\r\n");
    }

    #[test]
    fn ap_flash() {
        let cmd = QGpsCfgAutoGpsSet::new(OnOff::On);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"apflash\",1\r\n");
        let cmd = QGpsCfgAutoGpsSet::new(OnOff::On);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSCFG=\"apflash\",0\r\n");
    }

    #[test]
    fn gps_del() {
        let cmd = QGpsDelSet::new(DeleteType::AllDel);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSDEL=0\r\n");
        let cmd = QGpsDelSet::new(DeleteType::NotDel);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSDEL=1\r\n");
        let cmd = QGpsDelSet::new(DeleteType::PartDel);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSDEL=2\r\n");
    }

    #[test]
    fn qgps() {
        let cmd = QGpsSet::new(OnOff::On);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPS=1\r\n");
        let cmd = QGpsSet::new(OnOff::Off);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPS=0\r\n");
    }

    #[test]
    fn gps_end() {
        let cmd = QGpsEndSet;
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSEND\r\n");
    }

    #[test]
    fn gps_loc() {
        let cmd = QGpsLocGet::default();
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSLOC=0\r\n");
    }

    #[test]
    fn gps_gnmea() {
        let cmd = QGpsNmeaGet::new(super::NmeaType::GGA);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSGNMEA=\"GGA\"\r\n");
        let cmd = QGpsNmeaGet::new(super::NmeaType::GLL);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSGNMEA=\"GLL\"\r\n");
        let cmd = QGpsNmeaGet::new(super::NmeaType::GSA);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSGNMEA=\"GSA\"\r\n");
        let cmd = QGpsNmeaGet::new(super::NmeaType::GSV);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSGNMEA=\"GSV\"\r\n");
        let cmd = QGpsNmeaGet::new(super::NmeaType::RMC);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSGNMEA=\"RMC\"\r\n");
        let cmd = QGpsNmeaGet::new(super::NmeaType::VTG);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QGPSGNMEA=\"VTG\"\r\n");
    }

    #[test]
    fn agps() {
        let cmd = QAgpsSet::new(OnOff::On);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QAGPS=1\r\n");
        let cmd = QAgpsSet::new(OnOff::Off);
        let b = cmd.as_bytes();
        assert_eq!(b, b"AT+QAGPS=0\r\n");
    }
}

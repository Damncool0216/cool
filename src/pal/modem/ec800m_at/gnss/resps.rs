use super::types::{NmeaStr, NmeaVec};
use atat::{
    atat_derive::AtatResp,
    digest::ParseError,
    heapless::String,
    helpers::LossyStr,
    nom::{bytes, character, combinator, sequence},
    Error,
};
use log::{debug, error};

/// NmeaResp
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct NmeaResp {
    pub nmeas: NmeaVec,
}

fn parse_nmea_sentence(input: &[u8]) -> Option<(NmeaStr, u16)> {
    #[cfg(feature = "debug")]
    {
        let str = LossyStr(input);
        debug!("{:?}", str);
    }

    let prefix = b"+QGPSGNMEA: ";
    let mut start = 0;

    if input.starts_with(b"\r\n+QGPSGNMEA: ") {
        start = start + 2;
    }
    let input = &input[start as usize..];

    if input.starts_with(prefix) {
        let (_, sentence) = input.split_at(prefix.len());
        let end = if let Some(i) = sentence.iter().position(|&c| c == b'\r') {
            i
        } else {
            sentence.len()
        };
        let mut result = NmeaStr::new();
        for &c in &sentence[..end] {
            result.push(c as char).ok()?;
        }
        return Some((result, (start + prefix.len() + end) as u16));
    }
    None
}
impl NmeaResp {
    pub fn parse(data: &[u8]) -> Result<NmeaResp, Error> {
        Ok(Self::from(data))
    }
}
impl From<&[u8]> for NmeaResp {
    fn from(data: &[u8]) -> Self {
        let mut nmeas = NmeaVec::new();
        let mut cur = 0u16;

        while let Some((sentence, len)) = parse_nmea_sentence(&data[cur as usize..]) {
            #[cfg(feature = "debug")]
            debug!("{}", sentence);
            nmeas.push(sentence).ok();
            cur = cur + len as u16;
        }

        NmeaResp { nmeas }
    }
}

/// NmeaResp
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct GpsLocResp {
    pub utc: String<30>, //字符串类型。UTC 时间。格式：hhmmss.ss（引自 GPGGA 语句）。
    pub latitude: f32,
    pub longitude: f32,
    pub hdop: f32,
    pub altitude: f32,
    pub fix: u8,
    pub cog: Option<f32>,
    pub spkm: f32,
    pub spkn: f32,
    pub date: String<30>,
    pub nsat: u8,
}

impl GpsLocResp {
    pub fn parse(data: &[u8]) -> Result<GpsLocResp, Error> {
        parse_gps_loc(data).map_err(|_e| Error::Parse)
    }
}

pub(crate) fn parse_gps_loc(buf: &[u8]) -> Result<GpsLocResp, ParseError> {
    debug!("start parse qmt pubex");
    let byte_to_str = |st| {
        core::str::from_utf8(st).map_err(|_e| {
            #[cfg(feature = "debug")]
            error!("Failed to parse byte to str, {:?}", st);
            ParseError::NoMatch
        })
    };

    let byte_to_f32 = |st| {
        byte_to_str(st)?.parse::<f32>().map_err(|_e| {
            #[cfg(feature = "debug")]
            error!("Failed to parse byte to str, {:?}", st);
            ParseError::NoMatch
        })
    };
    let get_parm = |s, b| {
        sequence::preceded(
            bytes::complete::tag(s),
            bytes::complete::take_till(move |f| f == b),
        )
    };
    let (_, (utc, latitude, longitude, hdop, altitude, fix, cog, spkm, spkn, date, nsat)) =
        sequence::tuple((
            get_parm("+QGPSLOC: ", b','),
            get_parm(",", b','),
            get_parm(",", b','),
            get_parm(",", b','),
            get_parm(",", b','),
            combinator::map_parser(get_parm(",", b','), character::complete::u8),
            get_parm(",", b','),
            get_parm(",", b','),
            get_parm(",", b','),
            get_parm(",", b','),
            combinator::map_parser(get_parm(",", b','), character::complete::u8),
        ))(buf)?;
    let utc = byte_to_str(utc)?;
    let latitude = byte_to_f32(latitude)?;
    let longitude = byte_to_f32(longitude)?;
    let hdop = byte_to_f32(hdop)?;
    let altitude = byte_to_f32(altitude)?;
    let fix: u8 = fix;
    let cog = if cog == &b""[..] {
        None
    } else {
        Some(byte_to_f32(cog)?)
    };
    let spkm = byte_to_f32(spkm)?;
    let spkn = byte_to_f32(spkn)?;
    let date = byte_to_str(date)?;
    let nsat: u8 = nsat;

    let res = GpsLocResp {
        utc: String::try_from(utc).unwrap(),
        latitude,
        longitude,
        hdop,
        altitude,
        fix,
        cog,
        spkm,
        spkn,
        date: String::try_from(date).unwrap(),
        nsat,
    };
    Ok(res)
}

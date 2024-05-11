use crate::pal::modem::ec800m_at::parse_num;

use super::types::{NmeaStr, NmeaVec};
use atat::{
    atat_derive::AtatResp,
    digest::ParseError,
    helpers::LossyStr,
    nom::{
        bytes::{self, complete::take},
        character,
        combinator::{self, map_parser, map_res},
        number::complete::double,
        sequence::{self, tuple},
        IResult,
    },
    Error,
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use core::str;
use log::{debug, error, info};

use num_traits::float::FloatCore;

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
    pub latitude: f32,
    pub longitude: f32,
    pub hdop: f32,
    pub altitude: f32,
    pub fix: u8,
    pub cog: Option<f32>,
    pub spkm: f32,
    pub spkn: f32,
    pub nsat: u8,
    pub timestamp: i64,
}

pub fn parse_hms(i: &str) -> IResult<&str, NaiveTime> {
    map_res(
        tuple((
            map_res(take(2usize), parse_num::<u32>),
            map_res(take(2usize), parse_num::<u32>),
            map_parser(take(5usize), double),
        )),
        |(hour, minutes, sec)| -> core::result::Result<NaiveTime, &'static str> {
            info!("{hour} {minutes} {sec}");
            if sec.is_sign_negative() {
                return Err("Invalid time: second is negative");
            }
            if hour >= 24 {
                return Err("Invalid time: hour >= 24");
            }
            if minutes >= 60 {
                return Err("Invalid time: min >= 60");
            }
            if sec >= 60. {
                return Err("Invalid time: sec >= 60");
            }
            NaiveTime::from_hms_nano_opt(
                hour,
                minutes,
                sec.trunc() as u32,
                (sec.fract() * 1_000_000_000f64).round() as u32,
            )
            .ok_or("Invalid time")
        },
    )(i)
}

pub(crate) fn parse_date(i: &str) -> IResult<&str, NaiveDate> {
    map_res(
        tuple((
            map_res(take(2usize), parse_num::<u8>),
            map_res(take(2usize), parse_num::<u8>),
            map_res(take(2usize), parse_num::<u8>),
        )),
        |data| -> Result<NaiveDate, &'static str> {
            let (day, month, year) = (u32::from(data.0), u32::from(data.1), i32::from(data.2));

            // We only receive a 2digit year code in this message, this has the potential
            // to be ambiguous regarding the year. We assume that anything above 83 is 1900's, and
            // anything above 0 is 2000's.
            //
            // The reason for 83 is that NMEA0183 was released in 1983.
            // Parsing dates from ZDA messages is preferred, since it includes a 4 digit year.
            let year = match year {
                83..=99 => year + 1900,
                _ => year + 2000,
            };

            if !(1..=12).contains(&month) {
                return Err("Invalid month < 1 or > 12");
            }
            if !(1..=31).contains(&day) {
                return Err("Invalid day < 1 or > 31");
            }
            NaiveDate::from_ymd_opt(year, month, day).ok_or("Invalid date")
        },
    )(i)
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
    let nsat: u8 = nsat;

    let utc = byte_to_str(utc)?;
    info!("utc{}", utc);
    let utc = parse_hms(utc).map_err(|_| ParseError::NoMatch)?.1;
    let date = byte_to_str(date)?;
    info!("date{}", date);
    let date = parse_date(date).map_err(|_| ParseError::NoMatch)?.1;

    let data_time = NaiveDateTime::new(date, utc);
    let timestamp = data_time.and_utc().timestamp();
    let res = GpsLocResp {
        latitude,
        longitude,
        hdop,
        altitude,
        fix,
        cog,
        spkm,
        spkn,
        nsat,
        timestamp,
    };
    Ok(res)
}

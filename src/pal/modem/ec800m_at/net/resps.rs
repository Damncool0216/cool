use crate::pal::modem::ec800m_at::parse_num;
use atat::{
    atat_derive::AtatResp,
    digest::ParseError,
    nom::{
        bytes::{
            self,
            complete::{take, take_till},
        },
        character,
        combinator::{self, map_res},
        sequence::{self, tuple},
        IResult,
    }, Error,
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use log::{debug, error, info};

pub fn parse_hms(i: &str) -> IResult<&str, NaiveTime> {
    map_res(
        tuple((
            map_res(take(2usize), parse_num::<u32>),
            take(1 as usize),
            map_res(take(2usize), parse_num::<u32>),
            take(1 as usize),
            map_res(take(2usize), parse_num::<u32>),
        )),
        |(hour, _, minutes, _, sec)| -> core::result::Result<NaiveTime, &'static str> {
            info!("{hour} {minutes} {sec}");
            if hour >= 24 {
                return Err("Invalid time: hour >= 24");
            }
            if minutes >= 60 {
                return Err("Invalid time: min >= 60");
            }
            if sec >= 60 {
                return Err("Invalid time: sec >= 60");
            }
            NaiveTime::from_hms_opt(hour, minutes, sec).ok_or("Invalid time")
        },
    )(i)
}

pub(crate) fn parse_date(i: &str) -> IResult<&str, NaiveDate> {
    map_res(
        tuple((
            map_res(take(4usize), parse_num::<i32>),
            take(1 as usize),
            map_res(take(2usize), parse_num::<u8>),
            take(1 as usize),
            map_res(take(2usize), parse_num::<u8>),
        )),
        |data| -> Result<NaiveDate, &'static str> {
            let (year, month, day) = (data.0, u32::from(data.2), u32::from(data.4));
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

#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct QLtsResp {
    pub utc_stamp: i64,
}
impl QLtsResp {
    pub fn parse(data: &[u8]) -> Result<QLtsResp, Error> {
        parse_qlts(data).map_err(|_e| Error::Parse)
    }
}

fn parse_qlts(buf: &[u8]) -> Result<QLtsResp, ParseError> {
    debug!("start parse qlts resp");
    let byte_to_str = |st| {
        core::str::from_utf8(st).map_err(|_e| {
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
    //yyyy/MM/dd,hh:mm:ss±zz
    //+QLTS: "2019/01/13,11:41:23+32,0"
    let (_, (date, _, time, _offset, _dst, _)) = sequence::tuple((
        get_parm("+QLTS: \"", b','), //yyyy/MM/dd
        take(1 as usize),            //,
        take(8 as usize),            //hh:mm:ss
        take_till(|f| f == b','),    //+-zz
        combinator::map_parser(get_parm(",", b'\"'), character::complete::u8),
        take(1 as usize),
    ))(buf)?;
    let date = byte_to_str(date)?;
    let time = byte_to_str(time)?;
    info!("date: [{}], time: [{}]", date, time);
    let date = parse_date(date).map_err(|_| ParseError::NoMatch)?.1;
    let time = parse_hms(time).map_err(|_| ParseError::NoMatch)?.1;
    let data_time = NaiveDateTime::new(date, time);
    let timestamp = data_time.and_utc().timestamp();
    Ok(QLtsResp { utc_stamp: timestamp })
}

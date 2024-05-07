//! # URC parser implementation
//!
//! This is just used internally, but needs to be public for passing [URCMessages] as a generic to
//! [AtDigester](atat::digest::AtDigester): `AtDigester<URCMessages>`.

#[cfg(feature = "debug")]
use crate::{debug, info};
use atat::digest::ParseError;
#[cfg(feature = "debug")]
use atat::helpers::LossyStr;
use atat::{
    nom::{branch, bytes, character, combinator, sequence},
    AtatUrc, Parser,
};
use function_name::named;
use log::error;

/// URC definitions, needs to passed as generic of [AtDigester](atat::digest::AtDigester): `AtDigester<URCMessages>`
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum URCMessages {
    /// Unknown URC message
    Unknown,
    QmtOpen {
        client_idx: u16,
        result: i8,
    },
    QmtClose,
    QmtConn {
        client_idx: u16,
        result: u8,
        ret_code: u8,
    },
    QmtPubex {
        client_idx: u16,
        result: u8,
        ret_code: u8,
    },
    CREG {
        stat: u8,
        lac: u16,
        ci: u32,
        act: u8,
    },
    QmtStat,
}

impl URCMessages {
    #[named]
    pub(crate) fn parse_qmt_open(buf: &[u8]) -> Result<URCMessages, ParseError> {
        debug!("start parse qmt open");
        let (_, (_, id, _, res)) = sequence::tuple((
            bytes::complete::tag("+QMTOPEN: "),
            combinator::map_parser(
                bytes::complete::take_while(character::is_digit),
                character::complete::u16,
            ),
            bytes::streaming::tag(","),
            combinator::map_parser(
                bytes::complete::take_while(character::is_digit),
                character::complete::i8,
            ),
        ))(buf)?;
        let urc = URCMessages::QmtOpen {
            client_idx: id,
            result: res,
        };
        debug!("{:?}", urc);
        Ok(urc)
    }

    #[named]
    pub(crate) fn parse_qmt_conn(buf: &[u8]) -> Result<URCMessages, ParseError> {
        debug!("start parse qmt conn");
        let (_, (_, id, _, res, _, ret)) = sequence::tuple((
            bytes::complete::tag("+QMTCONN: "),
            combinator::map_parser(
                bytes::complete::take_while(character::is_digit),
                character::complete::u16,
            ),
            bytes::streaming::tag(","),
            combinator::map_parser(
                bytes::complete::take_while(character::is_digit),
                character::complete::u8,
            ),
            bytes::streaming::tag(","),
            combinator::map_parser(
                bytes::complete::take_while(character::is_digit),
                character::complete::u8,
            ),
        ))(buf)?;
        let urc = URCMessages::QmtConn {
            client_idx: id,
            result: res,
            ret_code: ret,
        };
        debug!("{:?}", urc);
        Ok(urc)
    }

    #[named]
    pub(crate) fn parse_qmt_pubex(buf: &[u8]) -> Result<URCMessages, ParseError> {
        debug!("start parse qmt pubex");
        let (_, (_, id, _, res, _, ret)) = sequence::tuple((
            bytes::complete::tag("+QMTPUBEX: "),
            combinator::map_parser(
                bytes::complete::take_while(character::is_digit),
                character::complete::u16,
            ),
            bytes::streaming::tag(","),
            combinator::map_parser(
                bytes::complete::take_while(character::is_digit),
                character::complete::u8,
            ),
            bytes::streaming::tag(","),
            combinator::map_parser(
                bytes::complete::take_while(character::is_digit),
                character::complete::u8,
            ),
        ))(buf)?;
        let urc = URCMessages::QmtPubex {
            client_idx: id,
            result: res,
            ret_code: ret,
        };
        debug!("{:?}", urc);
        Ok(urc)
    }

    #[named]
    pub(crate) fn parse_cerg(buf: &[u8]) -> Result<URCMessages, ParseError> {
        debug!("start parse");
        //+CREG: 1,\"287E\",\"F3BD34D\",7
        let (_, (_, stat, _, lac, _, ci, _, act)) = sequence::tuple((
            bytes::complete::tag("+CREG: "),
            combinator::map_parser(
                bytes::complete::take_while(character::is_digit),
                character::complete::u8,
            ),
            bytes::complete::take(2 as usize),
            bytes::complete::take_while(character::is_hex_digit),
            bytes::complete::take(3 as usize),
            bytes::complete::take_while(character::is_hex_digit),
            bytes::complete::take(2 as usize),
            combinator::map_parser(
                bytes::complete::take_while(character::is_digit),
                character::complete::u8,
            ),
        ))(buf)?;

        let lac = core::str::from_utf8(lac).map_err(|_e| {
            #[cfg(feature = "debug")]
            error!("Failed to parse lac, {:?}", LossyStr(lac));
            ParseError::NoMatch
        })?;
        let lac = u16::from_str_radix(lac, 16).map_err(|_e| {
            #[cfg(feature = "debug")]
            error!("Failed to parse lac from str");
            ParseError::NoMatch
        })?;

        let ci = core::str::from_utf8(ci).map_err(|_e| {
            #[cfg(feature = "debug")]
            error!("Failed to parse ci, {:?}", LossyStr(ci));
            ParseError::NoMatch
        })?;
        let ci = u32::from_str_radix(ci, 16).map_err(|_e| {
            #[cfg(feature = "debug")]
            error!("Failed to parse ci from str");
            ParseError::NoMatch
        })?;

        let urc = URCMessages::CREG { stat, lac, ci, act };
        debug!("{:?}", urc);
        Ok(urc)
    }
}

impl AtatUrc for URCMessages {
    type Response = Self;
    fn parse(resp: &[u8]) -> Option<Self::Response> {
        match resp {
            b if b.starts_with(b"+QMTOPEN") => URCMessages::parse_qmt_open(resp).ok(),
            b if b.starts_with(b"+QMTCONN") => URCMessages::parse_qmt_conn(resp).ok(),
            b if b.starts_with(b"+CREG") => URCMessages::parse_cerg(resp).ok(),
            b if b.starts_with(b"+QMTPUBEX") => URCMessages::parse_qmt_pubex(resp).ok(),
            _ => None,
        }
    }
}

impl Parser for URCMessages {
    #[named]
    fn parse(buf: &[u8]) -> Result<(&[u8], usize), ParseError> {
        let (_reminder, (head, data, tail)) = branch::alt((
            // URC
            sequence::tuple((
                bytes::streaming::tag(b"\r\n+QIND: "),
                bytes::streaming::take_till(|c| c == b'\r'),
                bytes::streaming::tag(b"\r\n"),
            )),
            sequence::tuple((
                bytes::streaming::tag("\r\n"),
                combinator::recognize(sequence::tuple((
                    bytes::streaming::tag("+QMTOPEN: "),
                    bytes::streaming::take_till(|f| f == b','),
                    bytes::streaming::tag(","),
                    bytes::streaming::take_till(|f| f == b'\r'),
                ))),
                bytes::streaming::tag("\r\n"),
            )),
            sequence::tuple((
                bytes::streaming::tag("\r\n"),
                combinator::recognize(sequence::tuple((
                    bytes::streaming::tag("+QMTCONN: "),
                    bytes::streaming::take_till(|f| f == b','),
                    bytes::streaming::tag(","),
                    bytes::streaming::take_till(|f| f == b','),
                    bytes::streaming::tag(","),
                    bytes::streaming::take_till(|f| f == b'\r'),
                ))),
                bytes::streaming::tag("\r\n"),
            )),
            sequence::tuple((
                bytes::streaming::tag("\r\n"),
                combinator::recognize(sequence::tuple((
                    bytes::streaming::tag("+QMTPUBEX: "),
                    bytes::streaming::take_till(|f| f == b','),
                    bytes::streaming::tag(","),
                    bytes::streaming::take_till(|f| f == b','),
                    bytes::streaming::tag(","),
                    bytes::streaming::take_till(|f| f == b'\r'),
                ))),
                bytes::streaming::tag("\r\n"),
            )),
            sequence::tuple((
                bytes::streaming::tag("\r\n"),
                combinator::recognize(sequence::tuple((
                    bytes::streaming::tag("+CREG: "),
                    bytes::streaming::take_while(character::is_digit), //stat
                    bytes::streaming::tag(","),
                    bytes::streaming::take_till(|f| f == b','), //lac
                    bytes::streaming::tag(","),
                    bytes::streaming::take_till(|f| f == b','), //ci
                    bytes::streaming::tag(","),
                    bytes::streaming::take_while(character::is_digit), //act
                ))),
                bytes::streaming::tag("\r\n"),
            )),
        ))(buf)?;
        #[cfg(feature = "debug")]
        info!("Urc success ! [{:?}]", LossyStr(data));
        Ok((data, head.len() + data.len() + tail.len()))
    }
}

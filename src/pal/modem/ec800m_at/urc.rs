//! # URC parser implementation
//!
//! This is just used internally, but needs to be public for passing [URCMessages] as a generic to
//! [AtDigester](atat::digest::AtDigester): `AtDigester<URCMessages>`.

#[cfg(feature = "debug")]
use crate::{debug, info};
use atat::digest::parser;
#[cfg(feature = "debug")]
use atat::helpers::LossyStr;
use atat::{digest::ParseError, nom::FindSubstring};
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
    Agps {
        valid: bool,
    },
    Gnss {
        update: bool,
    },
    QmtStat,
    RDY,
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
        let (_, (id, res, ret)) = sequence::tuple((
            sequence::preceded(
                bytes::complete::tag("+QMTCONN: "),
                combinator::map_parser(
                    bytes::complete::take_while(character::is_digit),
                    character::complete::u16,
                ),
            ),
            sequence::preceded(
                bytes::complete::tag(","),
                combinator::map_parser(
                    bytes::complete::take_while(character::is_digit),
                    character::complete::u8,
                ),
            ),
            sequence::preceded(
                bytes::complete::tag(","),
                combinator::map_parser(
                    bytes::complete::take_while(character::is_digit),
                    character::complete::u8,
                ),
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
        let (_, (id, res, ret)) = sequence::tuple((
            sequence::preceded(
                bytes::complete::tag("+QMTPUBEX: "),
                combinator::map_parser(
                    bytes::complete::take_while(character::is_digit),
                    character::complete::u16,
                ),
            ),
            sequence::preceded(
                bytes::streaming::tag(","),
                combinator::map_parser(
                    bytes::complete::take_while(character::is_digit),
                    character::complete::u8,
                ),
            ),
            sequence::preceded(
                bytes::streaming::tag(","),
                combinator::map_parser(
                    bytes::complete::take_while(character::is_digit),
                    character::complete::u8,
                ),
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
        let byte_to_str = |st| {
            core::str::from_utf8(st).map_err(|_e| {
                #[cfg(feature = "debug")]
                error!("Failed to parse byte to str, {:?}", st);
                ParseError::NoMatch
            })
        };
        let hex_str_to_u32 = |st| {
            u32::from_str_radix(byte_to_str(st)?, 16).map_err(|_e| {
                #[cfg(feature = "debug")]
                error!("Failed to parse num from str: {:?}", st);
                ParseError::NoMatch
            })
        };
        let (_, (a, b, lac, ci, act)) = sequence::tuple((
            sequence::preceded(
                bytes::complete::tag("+CREG: "),
                combinator::map_parser(
                    bytes::complete::take_while(character::is_digit),
                    character::complete::u8,
                ),
            ),
            branch::alt((
                combinator::map_parser(
                    sequence::preceded(
                        bytes::complete::tag(","),
                        bytes::complete::take_while(character::is_digit),
                    ),
                    character::complete::i8,
                ),
                combinator::success(-1),
            )),
            sequence::delimited(
                bytes::complete::tag(",\""),
                bytes::complete::take_while(character::is_hex_digit),
                bytes::complete::tag("\","),
            ),
            sequence::delimited(
                bytes::complete::tag("\""),
                bytes::complete::take_while(character::is_hex_digit),
                bytes::complete::tag("\","),
            ),
            combinator::map_parser(
                bytes::complete::take_while(character::is_digit),
                character::complete::u8,
            ),
        ))(buf)?;

        let lac = hex_str_to_u32(lac)?;
        let ci = hex_str_to_u32(ci)?;

        let urc = URCMessages::CREG {
            stat: if b == -1 { a } else { b as u8 },
            lac: lac as u16,
            ci,
            act,
        };
        debug!("{:?}", urc);
        Ok(urc)
    }
    #[named]
    pub(crate) fn parse_gnss_update(buf: &[u8]) -> Result<URCMessages, ParseError> {
        debug!("start parse");
        //GNSS Update successed!!!!
        let update = buf.find_substring("Update successed").is_some();
        let urc = URCMessages::Gnss { update };
        debug!("{:?}", urc);
        Ok(urc)
    }

    #[named]
    pub(crate) fn parse_agps_check(buf: &[u8]) -> Result<URCMessages, ParseError> {
        debug!("start parse");
        //AGPS Check Vailed!!!!
        let valid: bool = buf.find_substring("Check Vailed").is_some();
        let urc = URCMessages::Agps { valid };
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
            b if b.starts_with(b"GNSS") => URCMessages::parse_gnss_update(resp).ok(),
            b if b.starts_with(b"AGPS") => URCMessages::parse_agps_check(resp).ok(),
            b if b.starts_with(b"RDY") => Some(URCMessages::RDY),
            _ => None,
        }
    }
}

impl Parser for URCMessages {
    #[named]
    fn parse(buf: &[u8]) -> Result<(&[u8], usize), ParseError> {
        let (_reminder, (data, len)) = branch::alt((
            branch::alt((parser::urc_helper("RDY"),)),
            branch::alt((
                parser::urc_helper("+CFUN"),
                parser::urc_helper("+CPIN"),
                parser::urc_helper("+CREG"),
                parser::urc_helper("+CGREG"),
                parser::urc_helper("+CTZV"),
                parser::urc_helper("+CTZE"),
            )),
            parser::urc_helper("+QIND"),
            parser::urc_helper("+QUSIM"),
            branch::alt((
                parser::urc_helper("+QMTOPEN"),
                parser::urc_helper("+QMTCONN"),
                parser::urc_helper("+QMTPUBEX"),
                parser::urc_helper("+QMTSTAT"),
                parser::urc_helper("+QMTRECV"),
                parser::urc_helper("+QMTPING"),
            )),
            branch::alt((
                parser::urc_helper("GNSS Update successed!!!!"),
                parser::urc_helper("AGPS Check Vailed!!!!"),
            )),
        ))(buf)?;
        #[cfg(feature = "debug")]
        info!("Urc success! [{:?}]", LossyStr(data));
        Ok((data, len))
    }
}

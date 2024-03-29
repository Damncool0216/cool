#[cfg(feature = "debug")]
use atat::helpers::LossyStr;
use atat::{
    digest::{parser, ParseError},
    InternalError, Response,
};
use atat::{
    nom,
    nom::{branch, bytes, character, combinator, sequence},
    DigestResult, Digester, Parser,
};

use crate::urc::URCMessages;
#[cfg(feature = "debug")]
use heapless::String;
#[cfg(feature = "debug")]
use log::{debug, info};

#[derive(Default)]
pub struct Ec800mDigester {}

impl Ec800mDigester {
    pub fn custom_error(buf: &[u8]) -> Result<(&[u8], usize), ParseError> {
        let (_reminder, (head, data, tail)) = branch::alt((
            sequence::tuple((
                combinator::success(&b""[..]),
                bytes::streaming::tag(b"ERROR(-1)"),
                bytes::streaming::tag(b"\r\n"),
            )),
        ))(buf)?;
        #[cfg(feature = "debug")]
        debug!("Custom error {:?}", LossyStr(data));
        Ok((data, head.len() + data.len() + tail.len()))
    }

    pub fn custom_success(buf: &[u8]) -> Result<(&[u8], usize), ParseError> {
        #[cfg(feature = "debug")]
        debug!("Custom success start {:?}", LossyStr(buf));
        let (_reminder, (head, data, tail)) = branch::alt((
            // AT command
            sequence::tuple((
                bytes::streaming::tag(b"+AT: "),
                bytes::streaming::take_until("\r\n"),
                bytes::streaming::tag("\r\n"),
            )),
            // OK
            sequence::tuple((
                bytes::streaming::tag("\r\n"),
                bytes::streaming::tag(b"OK"),
                bytes::streaming::tag("\r\n"),
            )),
        ))(buf)?;
        #[cfg(feature = "debug")]
        info!("Custom success ! [{:?}]", LossyStr(data));
        Ok((data, head.len() + data.len() + tail.len()))
    }
}

impl Digester for Ec800mDigester {
    fn digest<'a>(&mut self, input: &'a [u8]) -> (DigestResult<'a>, usize) {
        #[cfg(feature = "debug")]
        let s = LossyStr(input);
        #[cfg(feature = "debug")]
        info!("Digesting: {:?}", s);

        // Incomplete. Eat the echo and do nothing else.
        let incomplete = (DigestResult::None, 0);

        // 2. Match for URC's
        match <URCMessages as Parser>::parse(input) {
            Ok((urc, len)) => return (DigestResult::Urc(urc), len),
            Err(ParseError::Incomplete) => return incomplete,
            _ => {}
        }

        // 3. Parse for success responses
        // Custom successful replies first, if any
        match (Ec800mDigester::custom_success)(input) {
            Ok((response, len)) => return (DigestResult::Response(Ok(response)), len),
            Err(ParseError::Incomplete) => return incomplete,
            _ => {}
        }

        // 4. Parse for error responses
        // Custom error matches first, if any
        match (Ec800mDigester::custom_error)(input) {
            Ok((response, len)) => {
                return (
                    DigestResult::Response(Err(InternalError::Custom(response))),
                    len,
                )
            }
            Err(ParseError::Incomplete) => return incomplete,
            _ => {}
        }

        // Generic error matches
        if let Ok((_, (result, len))) = parser::error_response(input) {
            return (result, len);
        }

        // No matches at all.
        incomplete
    }
}

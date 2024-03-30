use super::types::{NmeaStr, NmeaVec};
use atat::{
    atat_derive::AtatResp, helpers::LossyStr, serde_at::serde::Deserialize, AtatResp, Error,
    InternalError, Response,
};
use heapless::{String, Vec};
use log::{debug, info};

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

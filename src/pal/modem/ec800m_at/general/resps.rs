use atat::{atat_derive::AtatResp, heapless, Error};

use heapless::String;
use log::debug;

#[derive(Debug, Clone, AtatResp)]
pub struct NoResp;

/// OK resp
#[derive(Debug, Clone, AtatResp)]
pub struct OkResp {
    pub ok: String<4>,
}

impl OkResp {
    pub fn is_ok(&self) -> bool {
        self.ok.as_str().eq("OK")
    }
}

#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct SendResp {
    pub send_ready: String<2>,
}

impl SendResp {
    pub fn is_send_ready(&self) -> bool {
        self.send_ready.as_str().eq(">")
    }

    pub fn parse(data: &[u8]) -> Result<SendResp, Error> {
        debug!("parse SendResp {:?}", data);
        Ok(SendResp {
            send_ready: String::try_from(">").unwrap(),
        })
    }
}

/// ON/OFF resp
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct OnOffResp {
    pub on_off: String<6>,
}

impl OnOffResp {
    pub fn is_on(&self) -> bool {
        self.on_off.as_str().eq("ON")
    }
    pub fn is_off(&self) -> bool {
        self.on_off.as_str().eq("OFF")
    }
}

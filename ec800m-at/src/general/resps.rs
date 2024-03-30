use atat::{atat_derive::AtatResp, derive, serde_at::serde::Deserialize};

use heapless::{String, Vec};

#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct NoResp;

/// OK resp
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct OkResp {
    pub ok: String<2>,
}

impl OkResp {
    pub fn is_ok(&self) -> bool {
        self.ok.as_str().eq("OK")
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
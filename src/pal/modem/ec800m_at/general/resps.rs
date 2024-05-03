use atat::atat_derive::AtatResp;

use heapless::String;

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
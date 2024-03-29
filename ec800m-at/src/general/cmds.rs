use atat::atat_derive::AtatCmd;
use heapless::String;

use super::resps::{OkResp, OnOffResp};

/// AT - Verify COM is working
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("AT", OkResp, cmd_prefix = "", timeout_ms = 5000)]
pub struct VerifyAT;
///  Get ATE - Echo is on/off
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+ATE=?", OnOffResp)]
pub struct AteGet;

///  Set ATE - Echo is on/off
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("ATE", OkResp, cmd_prefix = "", timeout_ms = 5000, value_sep = false)]
pub struct AteSet {
    on_off: u8
}
impl AteSet {
    pub fn on() -> Self {
        Self {
            on_off: 1
        }
    }
    pub fn off() -> Self {
        Self {
            on_off: 0
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::general::cmds::{AteSet, VerifyAT};
    use atat::AtatCmd;

    #[test]
    fn verify_com_is_working() {
        let k = VerifyAT;
        let k = k.as_bytes();
        assert_eq!(k, b"AT\r\n");
    }

    #[test]
    fn ate_set() {
        let k = AteSet::on();
        assert_eq!(k, b"ATE1\r\n");
        let k = AteSetOff::off().as_bytes();
        assert_eq!(k, b"ATE0\r\n");
    }
}


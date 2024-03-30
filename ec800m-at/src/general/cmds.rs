use atat::atat_derive::AtatCmd;
use heapless::String;

use super::{
    resps::{OkResp, OnOffResp},
    types::OnOff,
};

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
#[at_cmd("ATE", OkResp, cmd_prefix = "", timeout_ms = 500, value_sep = false)]
pub struct AteSet {
    on_off: u8,
}
impl AteSet {
    pub fn new(on_off: OnOff) -> Self {
        Self {
            on_off: on_off as u8,
        }
    }
}

/// 2.12. AT&W 存储当前设置到用户自定义配置文件
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("AT&W0", OkResp, cmd_prefix = "", timeout_ms = 500, value_sep = false)]
pub struct AtW;

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

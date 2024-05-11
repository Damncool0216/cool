use crate::pal::modem::ec800m_at::general::resps::{OkResp, SendResp};
use atat::atat_derive::{AtatCmd, AtatResp};
use atat::heapless::String;

use super::resps::QLtsResp;
/// CopsResp
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct CposResp {
    ber: u8,
    err: u8,
}
//6.1. AT+COPS 选择运营商
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+COPS?", CposResp, timeout_ms = 15000)]
pub struct CopsQuery;

//6.2. AT+CREG CS 域网络注册状态
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct CregResp {
    n: u8,
    pub stat: u8,
    pub lac: Option<String<4>>,
    pub ci: Option<String<10>>,
    pub act: Option<u8>,
}

impl CregResp {
    pub fn is_attached(&self) -> bool {
        self.stat == 1 || self.stat == 5
    }
}
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CREG?", CregResp, timeout_ms = 300)]
pub struct CregQuery;

#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CREG", OkResp, timeout_ms = 300)]
pub struct CregSet {
    pub n: u8,
}

// 9.2 AT+CMGF 配置短消息模式
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CMGF", OkResp, timeout_ms = 600)]
pub struct CmgfSet {
    pub mode: u8,
}
impl CmgfSet {
    pub fn default() -> Self {
        Self {
            mode: 1
        }
    }
}

// 9.8 AT+CMGS 发送短消息
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CMGS", SendResp, timeout_ms = 3000, parse = SendResp::parse)]
pub struct CmgsSet<'a>{
    #[at_arg(position = 1, len = 64)]
    pub da: &'a str,
}

//6.3. AT+CSQ 查询信号强度

//6.4. AT+CPOL 配置首选运营商列表

//6.5. AT+COPN 查询运营商名称列表

//6.6. AT+CTZU 自动更新时区

//6.7. AT+CTZR 上报时区变化

//6.8. AT+QLTS 获取通过网络同步的最新时间

//6.9. AT+QNWINFO 查询网络信息

//5.3. AT+CPIN PIN 管理
#[derive(Debug, Clone, AtatResp, PartialEq)]
pub struct CpinResp {
    pub code: String<20>,
}
impl CpinResp {
    pub fn is_ready(&self) -> bool {
        self.code.eq("READY")
    }
}
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+CPIN?", CpinResp, timeout_ms = 300)]
pub struct CpinQuery;


//6.8. AT+QLTS 获取通过网络同步的最新时间
//AT+QLTS=1 查询通过网络同步的最新时间计算出的当前 GMT 时间。
//+QLTS: "2024/05/11,20:33:38+32,0"␍␊
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QLTS", QLtsResp, timeout_ms = 3000, parse = QLtsResp::parse)]
pub struct QLts {
    mode: u8
}
impl QLts {
    pub fn default() -> Self {
        Self {
            mode: 1
        }
    }
}

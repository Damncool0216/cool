use crate::general::{
    resps::{NoResp, OkResp},
    types::OnOff,
};
use atat::atat_derive::AtatCmd;
use heapless::String;

use super::types::{
    MqttCleanSession, MqttClientIdx, MqttEditMode, MqttPdPCid, MqttQos, MqttRecvLen, MqttRecvMode, MqttSendMode, MqttSslCtxIdx, MqttSslMode, MqttTimeOutNotice, MqttVersion, MqttWillFlag, MqttWillRetain
};

/// 3.3.1. AT+QMTCFG 配置 MQTT 可选参数
/// 配置 MQTT 协议版本 AT+QMTCFG="version",<client_idx>[,<vsn>]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QMTCFG", OkResp, timeout_ms = 600)]
pub struct QMtCfgVersionSet {
    #[at_arg(position = 1)]
    cfg: String<14>,
    #[at_arg(position = 2)]
    client_idx: u8,
    #[at_arg(position = 3)]
    vsn: u8,
}

impl QMtCfgVersionSet {
    pub fn new(client_idx: MqttClientIdx, vsn: MqttVersion) -> Self {
        Self {
            cfg: String::try_from("version").unwrap(),
            client_idx: client_idx as u8,
            vsn: vsn as u8,
        }
    }
}

///配置 MQTT 客户端待使用的 PDP AT+QMTCFG="pdpcid",<client_idx>[,<cid>]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QMTCFG", OkResp, timeout_ms = 600)]
pub struct QMtCfgPdpcidSet {
    #[at_arg(position = 1)]
    cfg: String<12>,
    #[at_arg(position = 2)]
    client_idx: u8,
    #[at_arg(position = 3)]
    cid: u8,
}
impl QMtCfgPdpcidSet {
    pub fn new(client_idx: MqttClientIdx, cid: MqttPdPCid) -> Self {
        Self {
            cfg: String::try_from("pdpcid").unwrap(),
            client_idx: client_idx as u8,
            cid: cid as u8,
        }
    }
}

/// 配置 MQTT SSL 模式和SSL 上下文索引AT+QMTCFG="ssl",<client_idx>[,<SSL_enable>[,<SSL_ctx_idx>]]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QMTCFG", OkResp, timeout_ms = 600)]
pub struct QMtCfgSslSet {
    #[at_arg(position = 1)]
    cfg: String<6>,
    #[at_arg(position = 2)]
    client_idx: u8,
    #[at_arg(position = 3)]
    ssl_enable: u8,
    #[at_arg(position = 4)]
    ssl_ctx_idx: u8,
}
impl QMtCfgSslSet {
    pub fn new(
        client_idx: MqttClientIdx,
        ssl_enable: MqttSslMode,
        ssl_ctx_idx: MqttSslCtxIdx,
    ) -> Self {
        Self {
            cfg: String::try_from("ssl").unwrap(),
            client_idx: client_idx as u8,
            ssl_enable: ssl_enable as u8,
            ssl_ctx_idx: ssl_ctx_idx as u8,
        }
    }
}

/// 配置保活时间 AT+QMTCFG="keepalive",<client_idx>[,<keep_alive_time>]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QMTCFG", OkResp, timeout_ms = 600)]
pub struct QMtCfgKeepAliveSet {
    #[at_arg(position = 1)]
    cfg: String<18>,
    #[at_arg(position = 2)]
    client_idx: u8,
    #[at_arg(position = 3)]
    keep_alive_time: u16,
}

impl QMtCfgKeepAliveSet {
    pub fn new(client_idx: MqttClientIdx, keep_alive_time: u16) -> Self {
        let keep_alive_time = if keep_alive_time > 3600 {
            120
        } else {
            keep_alive_time
        };
        Self {
            cfg: String::try_from("keepalive").unwrap(),
            client_idx: client_idx as u8,
            keep_alive_time,
        }
    }
}

/// 配置会话类型AT+QMTCFG="session",<client_idx>[,<clean_session>]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QMTCFG", OkResp, timeout_ms = 600)]
pub struct QMtCfgSessionSet {
    #[at_arg(position = 1)]
    cfg: String<14>,
    #[at_arg(position = 2)]
    client_idx: u8,
    #[at_arg(position = 3)]
    clean_session: u8,
}

impl QMtCfgSessionSet {
    pub fn new(client_idx: MqttClientIdx, clean_session: MqttCleanSession) -> Self {
        Self {
            cfg: String::try_from("session").unwrap(),
            client_idx: client_idx as u8,
            clean_session: clean_session as u8,
        }
    }
}

/// 配置消息传输超时时间AT+QMTCFG="timeout",<client_idx>[,<pkt_timeout>,<retry_times>,<timeout_notice>]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QMTCFG", OkResp, timeout_ms = 600)]
pub struct QMtCfgTimeOutSet {
    #[at_arg(position = 1)]
    cfg: String<14>,
    #[at_arg(position = 2)]
    client_idx: u8,
    #[at_arg(position = 3)]
    pkt_timeout: u8, //数据包传输超时时间 [1, 60]s 默认: 5
    #[at_arg(position = 4)]
    retry_times: u8, //数据包传输超时重发次数 [0-10] 默认: 3
    #[at_arg(position = 5)]
    time_out_notice: u8, //传输数据包时是否上报超时消息
}

impl QMtCfgTimeOutSet {
    pub fn new(
        client_idx: MqttClientIdx,
        pkt_timeout: u8,
        retry_times: u8,
        time_out_notice: MqttTimeOutNotice,
    ) -> Self {
        let pkt_timeout = if pkt_timeout < 1 || pkt_timeout > 60 {
            5
        } else {
            pkt_timeout
        };
        let retry_times = if retry_times > 10 { 3 } else { retry_times };
        Self {
            cfg: String::try_from("timeout").unwrap(),
            client_idx: client_idx as u8,
            pkt_timeout,
            retry_times,
            time_out_notice: time_out_notice as u8,
        }
    }
}

/// 配置 Will 信息
/// AT+QMTCFG="will",<client_idx>[,<will_fg>[,<will_qos>,<will_retain>,<willtopic>,<willmessage>]]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QMTCFG", OkResp, timeout_ms = 600)]
pub struct QMtCfgWillSet<'a> {
    #[at_arg(position = 1)]
    cfg: String<8>,
    #[at_arg(position = 2)]
    client_idx: u8,
    #[at_arg(position = 3)]
    will_fg: u8,
    #[at_arg(position = 4)]
    will_qos: u8,
    #[at_arg(position = 5)]
    will_retain: u8,
    #[at_arg(position = 6, len = 512)]
    will_topic: &'a str,
    #[at_arg(position = 7, len = 512)]
    will_message: &'a str,
}

impl<'a> QMtCfgWillSet<'a> {
    pub fn new(
        client_idx: MqttClientIdx,
        will_fg: MqttWillFlag,
        will_qos: MqttQos,
        will_reatin: MqttWillRetain,
        will_topic: &'a str,
        will_message: &'a str,
    ) -> Self {
        Self {
            cfg: String::try_from("will").unwrap(),
            client_idx: client_idx as u8,
            will_fg: will_fg as u8,
            will_qos: will_qos as u8,
            will_retain: will_reatin as u8,
            will_topic,
            will_message,
        }
    }
}

/// 配置 Will 信息
/// AT+QMTCFG="willex",<client_idx>[,<will_fg>[,<will_qos>,<will_retain>,<willtopic>,<will_len>]]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QMTCFG", OkResp, timeout_ms = 600)]
pub struct QMtCfgWillexSet<'a> {
    #[at_arg(position = 1)]
    cfg: String<12>,
    #[at_arg(position = 2)]
    client_idx: u8,
    #[at_arg(position = 3)]
    will_fg: u8,
    #[at_arg(position = 4)]
    will_qos: u8,
    #[at_arg(position = 5)]
    will_retain: u8,
    #[at_arg(position = 6, len = 512)]
    will_topic: &'a str,
    #[at_arg(position = 7)]
    will_len: u16,
}

impl<'a> QMtCfgWillexSet<'a> {
    pub fn new(
        client_idx: MqttClientIdx,
        will_fg: MqttWillFlag,
        will_qos: MqttQos,
        will_reatin: MqttWillRetain,
        will_topic: &'a str,
        will_len: u16,
    ) -> Self {
        Self {
            cfg: String::try_from("willex").unwrap(),
            client_idx: client_idx as u8,
            will_fg: will_fg as u8,
            will_qos: will_qos as u8,
            will_retain: will_reatin as u8,
            will_topic,
            will_len,
        }
    }
}

/// 配置服务器数据的接收模式
/// AT+QMTCFG="recv/mode",<client_idx>[,<msg_recv_mode>[,<msg_len_enable>]]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QMTCFG", OkResp, timeout_ms = 600)]
pub struct QMtCfgRecvModeSet {
    #[at_arg(position = 1)]
    cfg: String<18>,
    #[at_arg(position = 2)]
    client_idx: u8,
    #[at_arg(position = 3)]
    msg_recv_mode: u8,
    #[at_arg(position = 4)]
    msg_len_enable: u8,
}

impl QMtCfgRecvModeSet {
    pub fn new(
        client_idx: MqttClientIdx,
        msg_recv_mode: MqttRecvMode,
        msg_len_enable: MqttRecvLen,
    ) -> Self {
        Self {
            cfg: String::try_from("recv/mode").unwrap(),
            client_idx: client_idx as u8,
            msg_recv_mode: msg_recv_mode as u8,
            msg_len_enable: msg_len_enable as u8,
        }
    }
}

/// 配置阿里云设备信息
/// AT+QMTCFG="aliauth",<client_idx>[,<product key>,<devicename>,<device secret>]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QMTCFG", OkResp, timeout_ms = 600)]
pub struct QMtCfgAliauthSet<'a> {
    #[at_arg(position = 1)]
    cfg: String<14>,
    #[at_arg(position = 2)]
    client_idx: u8,
    #[at_arg(position = 3, len = 256)]
    product_key: &'a str,
    #[at_arg(position = 4, len = 128)]
    device_name: &'a str,
    #[at_arg(position = 5, len = 128)]
    device_secret: &'a str,
}

impl<'a> QMtCfgAliauthSet<'a> {
    pub fn new(
        client_idx: MqttClientIdx,
        product_key: &'a str,
        device_name: &'a str,
        device_secret: &'a str,
    ) -> Self {
        Self {
            cfg: String::try_from("aliauth").unwrap(),
            client_idx: client_idx as u8,
            product_key,
            device_name,
            device_secret,
        }
    }
}

/// 配置 MQTT 心跳间隔
/// AT+QMTCFG="qmtping",<client_idx>[,<qmtping_interval>]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QMTCFG", OkResp, timeout_ms = 600)]
pub struct QMtCfgQmtpingSet {
    #[at_arg(position = 1)]
    cfg: String<14>,
    #[at_arg(position = 2)]
    client_idx: u8,
    #[at_arg(position = 3)]
    qmtping_interval: u8,
}

impl QMtCfgQmtpingSet {
    pub fn new(client_idx: MqttClientIdx, qmtping_interval: u8) -> Self {
        let qmtping_interval = if qmtping_interval < 5 || qmtping_interval > 60 {
            5
        } else {
            qmtping_interval
        };
        Self {
            cfg: String::try_from("qmtping").unwrap(),
            client_idx: client_idx as u8,
            qmtping_interval,
        }
    }
}

/// 配置 MQTT 消息的发送格式
/// AT+QMTCFG="send/mode",<client_idx>[,<send_mode>]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QMTCFG", OkResp, timeout_ms = 600)]
pub struct QMtCfgSendModeSet {
    #[at_arg(position = 1)]
    cfg: String<14>,
    #[at_arg(position = 2)]
    client_idx: u8,
    #[at_arg(position = 3)]
    send_mode: u8,
}

impl QMtCfgSendModeSet {
    pub fn new(client_idx: MqttClientIdx, send_mode: MqttSendMode) -> Self {
        Self {
            cfg: String::try_from("send/mode").unwrap(),
            client_idx: client_idx as u8,
            send_mode: send_mode as u8
        }
    }
}

/// 配置华为 IoT 平台的设备信息
/// AT+QMTCFG="hwauth",<client_idx>[,<product id>,<device secret>]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QMTCFG", OkResp, timeout_ms = 600)]
pub struct QMtCfgHwauthSet<'a> {
    #[at_arg(position = 1)]
    cfg: String<12>,
    #[at_arg(position = 2)]
    client_idx: u8,
    #[at_arg(position = 3, len = 256)]
    product_id: &'a str,
    #[at_arg(position = 4, len = 256)]
    device_secret: &'a str,
}

impl<'a> QMtCfgHwauthSet<'a> {
    pub fn new(client_idx: MqttClientIdx, product_id: &'a str, device_secret: &'a str) -> Self {
        Self {
            cfg: String::try_from("hwauth").unwrap(),
            client_idx: client_idx as u8,
            product_id,
            device_secret
        }
    }
}

/// 配置华为 IoT 平台的设备信息
/// AT+QMTCFG="hwprodid",<client_idx>[,<product id>,<product secret>,<nodeid>]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QMTCFG", OkResp, timeout_ms = 600)]
pub struct QMtCfgHwprodidSet<'a> {
    #[at_arg(position = 1)]
    cfg: String<12>,
    #[at_arg(position = 2)]
    client_idx: u8,
    #[at_arg(position = 3, len = 256)]
    product_id: &'a str,
    #[at_arg(position = 4, len = 256)]
    device_secret: &'a str,
    #[at_arg(position = 5, len = 256)]
    nodeid: &'a str,
}

impl<'a> QMtCfgHwprodidSet<'a> {
    pub fn new(client_idx: MqttClientIdx, product_id: &'a str, device_secret: &'a str, nodeid: &'a str) -> Self {
        Self {
            cfg: String::try_from("hwprodid").unwrap(),
            client_idx: client_idx as u8,
            product_id,
            device_secret,
            nodeid
        }
    }
}

/// 配置 MQTT 数据格式
/// AT+QMTCFG="dataformat",<client_idx>[,<send_mode>,<recv_mode>]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QMTCFG", OkResp, timeout_ms = 600)]
pub struct QMtCfgDataFormatSet {
    #[at_arg(position = 1)]
    cfg: String<20>,
    #[at_arg(position = 2)]
    client_idx: u8,
    #[at_arg(position = 3)]
    send_mode: u8,
    #[at_arg(position = 4)]
    recv_mode: u8,
}

impl QMtCfgDataFormatSet {
    pub fn new(client_idx: MqttClientIdx, send_mode: MqttSendMode, recv_mode: MqttRecvMode) -> Self {
        Self {
            cfg: String::try_from("dataformat").unwrap(),
            client_idx: client_idx as u8,
            send_mode: send_mode as u8,
            recv_mode: recv_mode as u8
        }
    }
}

/// 配置透传模式下 MQTT 数据的回显模式
/// AT+QMTCFG="view/mode",<client_idx>[,<view_mode>]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QMTCFG", OkResp, timeout_ms = 600)]
pub struct QMtCfgViewModeSet {
    #[at_arg(position = 1)]
    cfg: String<18>,
    #[at_arg(position = 2)]
    client_idx: u8,
    #[at_arg(position = 3)]
    view_mode: u8,
}

impl QMtCfgViewModeSet {
    pub fn new(client_idx: MqttClientIdx, view_mode: MqttViewMode) -> Self {
        Self {
            cfg: String::try_from("view/mode").unwrap(),
            client_idx: client_idx as u8,
            view_mode: view_mode as u8

        }
    }
}

/// 配置 MQTT 输入数据超时时间
/// AT+QMTCFG="edit/timeout",<client_idx>[,<edit_mode>,<edit_time>]
#[derive(Clone, Debug, AtatCmd)]
#[at_cmd("+QMTCFG", OkResp, timeout_ms = 600)]
pub struct QMtCfgEditTimeoutSet {
    #[at_arg(position = 1)]
    cfg: String<24>,
    #[at_arg(position = 2)]
    client_idx: u8,
    #[at_arg(position = 3)]
    edit_mode: u8,
    #[at_arg(position = 4)]
    edit_time: u8,
}

impl QMtCfgEditTimeoutSet {
    pub fn new(client_idx: MqttClientIdx, edit_mode: MqttEditMode, edit_time: u8) -> Self {
        let edit_time = if edit_time < 1 || edit_time > 120 {30} else {edit_time};
        Self {
            cfg: String::try_from("edit/timeout").unwrap(),
            client_idx: client_idx as u8,
            edit_mode: edit_mode as u8,
            edit_time
        }
    }
}

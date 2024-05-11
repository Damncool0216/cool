use core::str::FromStr;

use atat::{AtatIngress, Ingress, ResponseSlot, UrcChannel};
use ec800m_at::{
    client::asynch::Ec800mClient, digester::Ec800mDigester, general::types::OnOff, urc::URCMessages,
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel,
    mutex::Mutex,
    pubsub::{Subscriber, WaitResult},
};
use embassy_time::Duration;
use function_name::named;
use hal::{
    peripherals::UART1,
    uart::{UartRx, UartTx},
    Async,
};

use serde_json_core::heapless::String;
use static_cell::StaticCell;

use crate::{
    debug,
    fml::{net::fml_net_send_dequeue, storage::*},
    info,
    pal::{
        self,
        modem::ec800m_at::mqtt::types::{MqttClientIdx, MqttQos, MqttVersion},
    },
};

use super::{Msg, MsgQueue};

mod ec800m_at;

const RX_SIZE: usize = 2048;
const INGRESS_BUF_SIZE: usize = RX_SIZE;
const URC_SUBSCRIBERS_NUM: usize = 1;
const URC_CAPACITY: usize = RX_SIZE * 3;

type AtReader<'a> = UartRx<'a, UART1, Async>;
type AtWriter<'a> = UartTx<'a, UART1, Async>;

type AtDigester = Ec800mDigester;
type AtClient<'a> = Ec800mClient<'a, AtWriter<'a>, INGRESS_BUF_SIZE>;

static RES_SLOT: ResponseSlot<INGRESS_BUF_SIZE> = ResponseSlot::new();

static PAL_MODEM_TASK_QUEUE: MsgQueue<50> = channel::Channel::new();

#[inline]
pub(super) async fn msg_req(msg: Msg) {
    PAL_MODEM_TASK_QUEUE.send(msg).await
}

#[embassy_executor::task()]
#[allow(unused_macros)]
#[named]
pub(super) async fn pal_at_client_task(writer: AtWriter<'static>) {
    // static NMEAPARSER: StaticCell<Nmea> = StaticCell::new();
    // let nmea_parser = NMEAPARSER
    //     .init(Nmea::create_for_navigation(&[SentenceType::RMC, SentenceType::GGA]).unwrap());
    static BUF: StaticCell<[u8; 1024]> = StaticCell::new();
    let mut config = atat::Config::default();
    config = config.flush_timeout(Duration::from_millis(500));
    let client = atat::asynch::Client::new(writer, &RES_SLOT, BUF.init([0; 1024]), config);
    let mut at_client = AtClient::new(client).await.unwrap();
    while let Err(_) = at_client.verify_com_is_working().await {
        debug!("At baudrate adapting!");
    }

    debug!("At client init ok!");
    at_client.at_echo_set(OnOff::Off).await.ok();
    at_client.urc_port_config().await.ok();
    at_client.creg_set(2).await.ok();
    at_client.mqtt_close(MqttClientIdx::IDX1).await.ok();
    at_client.cmgf_set(1).await.ok();
    at_client.at_config_save().await.ok();
    at_client.gps_set_sw(OnOff::Off).await.ok();
    loop {
        let msg = PAL_MODEM_TASK_QUEUE.receive().await;
        info!("{:?}", msg);
        match msg {
            Msg::ModemInitReq => {
                at_client.at_echo_set(OnOff::Off).await.ok();
                at_client.urc_port_config().await.ok();
                at_client.creg_set(2).await.ok();
                at_client.mqtt_close(MqttClientIdx::IDX1).await.ok();
                at_client.cmgf_set(1).await.ok();
                at_client.at_config_save().await.ok();
                at_client.gps_set_sw(OnOff::Off).await.ok();
            }
            Msg::GnssOpenReq => {
                let res = at_client.gps_set_sw(OnOff::On).await;
                info!("gnss open: {:?}", res);
                pal::msg_rpy(Msg::GnssOpenRpy(res.is_ok())).await
            }
            Msg::GnssCloseReq => {
                pal::msg_rpy(Msg::GnssCloseRpy(
                    at_client.gps_set_sw(OnOff::On).await.is_ok(),
                ))
                .await
            }
            Msg::GnssGetLocationReq => {
                if let Ok(s) = at_client.gps_get_location().await {
                    pal::msg_rpy(Msg::GnssGetLoactionRpy(Some(FmlGnssRawData {
                        latitude: s.latitude,
                        longitude: s.longitude,
                        hdop: s.hdop,
                        altitude: s.altitude,
                        fix: s.fix,
                        cog: s.cog,
                        spkm: s.spkm,
                        spkn: s.spkn,
                        nsat: s.nsat,
                        utc_stamp: s.timestamp,
                    })))
                    .await
                } else {
                    pal::msg_rpy(Msg::GnssGetLoactionRpy(None)).await
                }
            }
            Msg::NetAttachStatReq(s) => {
                if let Some(n) = s {
                    at_client.creg_set(n).await.ok();
                } else if let Ok((a, b, c, d)) = at_client.creg_query().await {
                    pal::msg_rpy(Msg::NetAttachStatRpy {
                        stat: a,
                        lac: b,
                        ci: c,
                        act: d,
                    })
                    .await;
                }
            }
            Msg::NetSimStatReq => {
                if let Ok(s) = at_client.sim_query().await {
                    pal::msg_rpy(Msg::NetSimStatRpy(s)).await
                }
            }
            Msg::NetGetTimeReq => {
                if let Ok(t) = at_client.qlts_set().await {
                    pal::msg_rpy(Msg::NetGetTimeRpy(t)).await
                }
            }
            Msg::MqttOpenReq => {
                let mut server = None;
                if let Some(net_nvm) = &*FML_NET_NVM.lock().await {
                    server = Some(net_nvm.mqtt_server.clone());
                }
                at_client
                    .mqtt_version_config(MqttClientIdx::IDX1, MqttVersion::V3_1_1)
                    .await
                    .ok();
                if let Some(server) = server {
                    at_client
                        .mqtt_open(MqttClientIdx::IDX1, &server.0, server.1)
                        .await
                        .ok();
                }
            }

            Msg::MqttCloseReq => {
                at_client.mqtt_close(MqttClientIdx::IDX1).await.ok();
            }

            Msg::MqttConnReq => {
                let mut info = None;
                if let Some(net_nvm) = &*FML_NET_NVM.lock().await {
                    info = Some((
                        net_nvm.mqtt_client_id.clone(),
                        net_nvm.mqtt_usename.clone(),
                        net_nvm.mqtt_password.clone(),
                    ));
                }
                if let Some(info) = info {
                    if let Ok(_s) = at_client
                        .mqtt_conn(MqttClientIdx::IDX1, &info.0, &info.1, &info.2)
                        .await
                    {
                        //pal::msg_rpy(Msg::MqttConnRpy(s)).await
                    } else {
                        pal::msg_rpy(Msg::MqttConnRpy(false)).await
                    }
                }
            }
            Msg::MqttPubReq => {
                let mut topic = String::new();
                if let Some(fml_net_nvm) = &*FML_NET_NVM.lock().await {
                    topic = fml_net_nvm.dp_topic.clone();
                }
                if let Some(s) = fml_net_send_dequeue() {
                    let data;
                    match s {
                        FmlNetSendType::TempHumi(t) => {
                            data = serde_json_core::ser::to_string::<FmlTempHumiData, 512>(&t)
                                .unwrap()
                                .clone();
                        }
                        FmlNetSendType::Location(l) => {
                            data = serde_json_core::to_string::<FmlGnssData, 512>(&l)
                                .unwrap()
                                .clone();
                        }
                    }
                    at_client
                        .mqtt_publish(MqttClientIdx::IDX1, 0, MqttQos::Qos0, 0, topic, data)
                        .await
                        .ok();
                }
            }
            Msg::AlarmTempSendReq(temp) => {
                let mut phone = String::new();
                let mut buf= [0;50];
                let mut alarm_msg = String::from_str(format_no_std::show(
                    &mut buf,
                    format_args!("Temp Alarm!!! {}C", temp),
                ).unwrap()).unwrap();
                if let Some(fml_net_nvm) = &*FML_NET_NVM.lock().await {
                    if let Some(s) = &fml_net_nvm.phone {
                        phone = s.clone();
                    } else {
                        pal::msg_rpy(Msg::AlarmTempSendRpy(false)).await;
                        continue;
                    }
                    if let Some(s) = &fml_net_nvm.temp_alarm_msg {
                        alarm_msg = s.clone();
                    }
                }
                info!("temp alarm sms send");
                pal::msg_rpy(Msg::AlarmTempSendRpy(
                    at_client.cmgs_set(phone, alarm_msg).await.is_ok(),
                ))
                .await;
            }
            Msg::AlarmHumiSendReq(humi) => {
                let mut phone = String::new();
                let mut buf= [0;50];
                let mut alarm_msg = String::from_str(format_no_std::show(
                    &mut buf,
                    format_args!("Humi Alarm!!! {}%", humi),
                ).unwrap()).unwrap();
                
                if let Some(fml_net_nvm) = &*FML_NET_NVM.lock().await {
                    if let Some(s) = &fml_net_nvm.phone {
                        phone = s.clone();
                    } else {
                        pal::msg_rpy(Msg::AlarmHumiSendRpy(false)).await;
                        continue;
                    }
                    if let Some(s) = &fml_net_nvm.temp_alarm_msg {
                        alarm_msg = s.clone();
                    }
                }
                info!("humi alarm sms send");
                pal::msg_rpy(Msg::AlarmHumiSendRpy(
                    at_client.cmgs_set(phone, alarm_msg).await.is_ok(),
                ))
                .await;
            }
            _ => unreachable!(),
        }
    }
}
pub(crate) type MsgUrc = URCMessages;
pub(crate) type MsgUrcChannel = UrcChannel<MsgUrc, URC_CAPACITY, URC_SUBSCRIBERS_NUM>;
pub(crate) type MsgUrcSubscriber =
    Subscriber<'static, CriticalSectionRawMutex, MsgUrc, URC_CAPACITY, URC_SUBSCRIBERS_NUM, 1>;

static URC_SUB_INIT: StaticCell<MsgUrcSubscriber> = StaticCell::new();
static URC_SUB_QUEUE: Mutex<CriticalSectionRawMutex, Option<&'static mut MsgUrcSubscriber>> =
    Mutex::new(None);

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
pub(crate) async fn pal_at_urc_task() {
    let mut init = false;
    loop {
        if let Some(_) = &mut *URC_SUB_QUEUE.lock().await {
            init = true;
        }
        if init {
            break;
        } else {
            embassy_time::Timer::after_millis(20).await;
        }
    }
    info!("start");
    let sub = &mut *URC_SUB_QUEUE.lock().await;
    let sub = sub.take().unwrap();
    loop {
        let msg = sub.next_message().await;
        debug!("{:?}", msg);
        match msg {
            WaitResult::Lagged(_i) => {}
            WaitResult::Message(msg) => match msg {
                MsgUrc::QmtOpen {
                    client_idx: _,
                    result,
                } => {
                    pal::msg_rpy(Msg::MqttOpenRpy(result == 0 || result == 2)).await;
                }
                MsgUrc::CREG { stat, lac, ci, act } => {
                    pal::msg_rpy(Msg::NetAttachStatRpy {
                        stat,
                        lac: Some(lac),
                        ci: Some(ci),
                        act: Some(act),
                    })
                    .await;
                }
                MsgUrc::QmtConn {
                    client_idx: _,
                    result,
                    ret_code: _,
                } => {
                    pal::msg_rpy(Msg::MqttConnRpy(result == 0)).await;
                }
                MsgUrc::QmtPubex {
                    client_idx: _,
                    result,
                    ret_code: _,
                } => {
                    pal::msg_rpy(Msg::MqttPubRpy(result == 0)).await;
                }
                MsgUrc::RDY => {
                    pal::msg_req(Msg::ModemInitReq).await;
                    pal::msg_rpy(Msg::ModemReady).await;
                }
                _ => {}
            },
        }
    }
}

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
pub(super) async fn pal_at_ingress_task(mut reader: AtReader<'static>) {
    static INGRESS_BUF: StaticCell<[u8; INGRESS_BUF_SIZE]> = StaticCell::new();
    static URC_CHANNEL: MsgUrcChannel = UrcChannel::new();
    info!("start");
    {
        let sub = &mut *URC_SUB_QUEUE.lock().await;
        if let Ok(s) = URC_CHANNEL.subscribe() {
            *sub = Some(URC_SUB_INIT.init(s));
        }
        //release lock after init
    }

    let mut ingress = Ingress::new(
        AtDigester::default(),
        INGRESS_BUF.init([0; INGRESS_BUF_SIZE]),
        &RES_SLOT,
        &URC_CHANNEL,
    );

    ingress.read_from(&mut reader).await
}

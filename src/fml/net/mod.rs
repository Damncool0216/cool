use core::sync::atomic::{AtomicU8, Ordering};

use embassy_sync::channel;
use embassy_time::Duration;
use function_name::named;
use serde_json_core::heapless::spsc::Consumer;

use crate::{
    debug, info,
    pal::{self, Msg, MsgQueue},
};

use super::storage::*;

static FML_NET_ATTA_STATUS: AtomicU8 = AtomicU8::new(0);
static FML_NET_CONN_STATUS: AtomicU8 = AtomicU8::new(0);

static FML_NET_ATTA_QUEUE: MsgQueue<10> = channel::Channel::new();
static FML_NET_RECV_QUEUE: MsgQueue<10> = channel::Channel::new();
static FML_NET_SEND_QUEUE: MsgQueue<30> = channel::Channel::new();

pub fn fml_net_att_status_get() -> FmlNetAttachStatus {
    FML_NET_ATTA_STATUS.load(Ordering::Relaxed).into()
}

pub fn fml_net_conn_status_get() -> FmlNetConnStatus {
    FML_NET_CONN_STATUS.load(Ordering::Relaxed).into()
}

#[inline]
pub async fn fml_net_mqtt_open() {
    pal::msg_req(Msg::MqttOpenReq).await;
}

#[inline]
pub async fn fml_net_mqtt_connect() {
    pal::msg_req(Msg::MqttConnReq).await;
}

#[inline]
pub(crate) async fn fml_net_mqtt_pub_req() {
    FML_NET_SEND_QUEUE.send(Msg::MqttPubReq).await
}

#[named]
async fn fml_net_conn_status_set(new_status: FmlNetConnStatus, force: bool) {
    let old_status: FmlNetConnStatus = FML_NET_CONN_STATUS.load(Ordering::Relaxed).into();
    if old_status == new_status && force != true {
        return;
    }
    info!("conn_status: {:?}", new_status);
    match new_status {
        FmlNetConnStatus::Down => {}
        FmlNetConnStatus::Connecting => {
            fml_net_mqtt_open().await;
        }
        FmlNetConnStatus::Connected => {
            fml_net_mqtt_connect().await;
        }
        FmlNetConnStatus::Logined => {
            fml_net_mqtt_pub_req().await;
        }
        FmlNetConnStatus::NotConnect => {}
    }
    FML_NET_CONN_STATUS.store(new_status as u8, Ordering::Relaxed);
}

#[named]
async fn fml_net_att_status_set(new_status: FmlNetAttachStatus, force: bool) {
    let old_status: FmlNetAttachStatus = FML_NET_ATTA_STATUS.load(Ordering::Relaxed).into();
    if old_status == new_status && force == false {
        return;
    }
    FML_NET_ATTA_STATUS.store(new_status as u8, Ordering::Relaxed);
    info!("att_status: {:?}", new_status);
    match new_status {
        FmlNetAttachStatus::NoSim => {
            pal::msg_req(Msg::NetSimStatReq).await;
        }
        FmlNetAttachStatus::SimReady => {
            pal::msg_req(Msg::NetAttachStatReq(None)).await;
        }
        FmlNetAttachStatus::NoService => {
            pal::msg_req(Msg::NetAttachStatReq(None)).await;
        }
        FmlNetAttachStatus::InCall => {}
        FmlNetAttachStatus::GSMService => {
            if fml_net_conn_status_get() < FmlNetConnStatus::Connecting {
                fml_net_conn_status_set(FmlNetConnStatus::Connecting, false).await;
            }
        }
        FmlNetAttachStatus::LTEService => {
            if fml_net_conn_status_get() < FmlNetConnStatus::Connecting {
                fml_net_conn_status_set(FmlNetConnStatus::Connecting, false).await;
            }
        }
    }
}

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
/// handle net attach
pub(super) async fn fml_net_status_task() {
    let mut retry = 0;
    {
        *(FML_NET_NVM.lock().await) = Some(FmLNetNvm::default())
    }
    fml_net_att_status_set(FmlNetAttachStatus::NoSim, true).await;
    loop {
        let msg = FML_NET_ATTA_QUEUE.receive().await;
        debug!("{:?}", msg);
        match msg {
            Msg::NetAttachStatRpy { stat, lac, ci, act } => {
                info!("stat:{:?}", stat);
                if stat == 1 || stat == 5 {
                    if let (Some(lac), Some(ci), Some(act)) = (lac, ci, act) {
                        info!("lac:{:x}, ci:{:X}, act:{}", lac, ci, act);
                        if act > 3 {
                            fml_net_att_status_set(FmlNetAttachStatus::LTEService, false).await;
                        } else {
                            fml_net_att_status_set(FmlNetAttachStatus::GSMService, false).await;
                        }
                    } else {
                        fml_net_att_status_set(FmlNetAttachStatus::LTEService, false).await;
                    }
                } else {
                    fml_net_att_status_set(FmlNetAttachStatus::NoService, false).await;
                }
            }
            Msg::NetSimStatRpy(ready) => {
                if ready {
                    info!("sim ready");
                    retry = 0;
                    fml_net_att_status_set(FmlNetAttachStatus::SimReady, false).await;
                } else {
                    fml_net_att_status_set(FmlNetAttachStatus::NoSim, true).await;
                }
            }

            Msg::MqttOpenRpy(open) => {
                if open {
                    info!("mqtt open success");
                    retry = 0;
                    fml_net_conn_status_set(FmlNetConnStatus::Connected, false).await;
                } else {
                    retry = retry + 1;
                    if retry < 3 {
                        fml_net_conn_status_set(FmlNetConnStatus::Connecting, true).await;
                    }
                }
            }
            Msg::MqttConnRpy(conn) => {
                if conn {
                    info!("mqtt connect success");
                    retry = 0;
                    fml_net_conn_status_set(FmlNetConnStatus::Logined, false).await;
                } else {
                    retry = retry + 1;
                    if retry < 3 {
                        fml_net_conn_status_set(FmlNetConnStatus::Connected, true).await;
                    }
                }
            }
            _ => {}
        }
    }
}

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
/// handle net data send
pub(super) async fn fml_net_send_task(mut temp_consumer: Consumer<'static, FmlTempHumiData, 128>) {
    let mut retry = 0;
    let mut ticker = embassy_time::Ticker::every(Duration::from_secs(30));
    loop {
        let msg = FML_NET_SEND_QUEUE.receive().await;
        debug!("{:?}", msg);
        match msg {
            Msg::MqttPubReq => {
                if fml_net_conn_status_get() == FmlNetConnStatus::Logined {
                    if let Some(fml_net_nvm) = &mut *FML_NET_NVM.lock().await {
                        if let Some(_) = fml_net_nvm.send_type {
                            info!("sending wait!");
                            continue;
                        }
                        if temp_consumer.ready() {
                            if let Some(s) = temp_consumer.peek() {
                                info!("pub temp humi");
                                fml_net_nvm.send_type =
                                    Some(FmlNetSendType::TempHumi((*s).clone()));
                                retry = 0;
                            }
                        }
                    }
                    pal::msg_req(Msg::MqttPubReq).await;
                }
            }
            Msg::MqttPubRpy(finish) => {
                if finish {
                    retry = 0;
                    if let Some(fml_net_nvm) = &mut *FML_NET_NVM.lock().await {
                        if let Some(send_type) = &fml_net_nvm.send_type {
                            match send_type {
                                FmlNetSendType::TempHumi(..) => {
                                    temp_consumer.dequeue();
                                    info!(
                                        "dequeue temp humi: {}/{}",
                                        temp_consumer.len(),
                                        temp_consumer.capacity()
                                    );
                                }
                                _ => {}
                            }
                        }
                        fml_net_nvm.send_type = None;
                    }
                    if temp_consumer.ready() {
                        fml_net_mqtt_pub_req().await;
                    }
                } else {
                    if retry < 3 {
                        retry = retry + 1;
                        fml_net_mqtt_pub_req().await;
                    } else {
                        if let Some(fml_net_nvm) = &mut *FML_NET_NVM.lock().await {
                            fml_net_nvm.send_type = None;
                        }
                    }
                }
            }
            _ => {}
        }
        ticker.next().await;
    }
}

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
/// handle net data recv
pub(super) async fn fml_net_recv_task() {
    loop {
        embassy_time::Timer::after_secs(60).await;
    }
}

#[inline]
pub(super) async fn msg_rpy(msg: Msg) {
    match msg {
        Msg::NetAttachStatRpy { .. } => FML_NET_ATTA_QUEUE.send(msg).await,
        Msg::NetSimStatRpy(..) => FML_NET_ATTA_QUEUE.send(msg).await,
        Msg::MqttOpenRpy(..) | Msg::MqttConnRpy(..) | Msg::MqttCloseRpy(..) => {
            FML_NET_ATTA_QUEUE.send(msg).await
        }
        Msg::MqttPubRpy(..) => FML_NET_SEND_QUEUE.send(msg).await,
        _ => {}
    }
}

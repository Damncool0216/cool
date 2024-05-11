use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel, mutex::Mutex};
use embassy_time::{Duration, Instant};
use function_name::named;

use crate::{
    debug,
    fml::{self, storage::*},
    info,
    pal::{self, Msg, MsgQueue},
};

use super::{net::fml_net_att_status_get, FmlDataProducer};

static FML_TEMP_MSG_QUEUE: MsgQueue<10> = channel::Channel::new();

#[inline]
pub async fn fml_temp_humi_get_req() {
    pal::msg_req(pal::Msg::TsensorGetReq).await
}

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
pub(super) async fn fml_temp_detect_task() {
    let mut detect_inv = 1;
    let mut ticker = embassy_time::Ticker::every(Duration::from_secs(detect_inv as u64 * 60));
    loop {
        if let Ok(fml_temp_nvm) = FML_TEMP_NVM.try_lock() {
            // not await in lock
            if let Some(fml_temp_nvm) = &*fml_temp_nvm {
                debug!("{:?}", fml_temp_nvm);
                if fml_temp_nvm.detect_inv != detect_inv {
                    ticker = embassy_time::Ticker::every(Duration::from_secs(
                        fml_temp_nvm.detect_inv as u64 * 60,
                    ));
                    detect_inv = fml_temp_nvm.detect_inv;
                }
            }
        }
        if detect_inv > 0 {
            fml_temp_humi_get_req().await;
        }
        ticker.next().await;
    }
}

#[inline]
pub(super) async fn msg_rpy(msg: Msg) {
    FML_TEMP_MSG_QUEUE.send(msg).await
}
pub struct FmlTempHumiAlarmConfig {
    pub low_temp: Option<f32>,
    pub high_temp: Option<f32>,
    pub low_humi: Option<f32>,
    pub high_humi: Option<f32>,
}

static TEMP_HUMI_ALARM: Mutex<CriticalSectionRawMutex, Option<FmlTempHumiAlarmConfig>> =
    Mutex::new(Some(FmlTempHumiAlarmConfig {
        low_temp: None,
        high_temp: None, //Some(-18.0),
        low_humi: None,
        high_humi: None,//Some(80.0),
    }));

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
pub(super) async fn fml_temp_msg_rpy_task(mut producer: FmlDataProducer<'static, FmlTempHumiData>) {
    let mut temp_alarm_slient_time: Option<Instant> = None;
    let mut humi_alarm_slient_time: Option<Instant> = None;
    loop {
        let msg = FML_TEMP_MSG_QUEUE.receive().await;
        info!("{:?}", msg);
        match msg {
            Msg::TsensorGetRpy { temp, humi } => {
                let t = fml::fml_system_time_get_sec().await;
                if t == 0 {
                    info!("wait for systime");
                    continue;
                }
                let data = FmlTempHumiData::new(1, temp, humi, t);
                producer.enqueue(data).ok();
                info!("temp humi enqueue!!! {}/{} at {}", producer.len(), producer.capacity(), t);
                if fml_net_att_status_get() < FmlNetAttachStatus::GSMService {
                    continue;
                }
                super::net::fml_net_mqtt_pub_req().await;
                //start check alarm 
                let mut temp_alarm = false;
                let mut humi_alarm = false;
                if let Some(temp_humi_alarm) = &*TEMP_HUMI_ALARM.lock().await {
                    if temp_humi_alarm.low_temp.is_some()
                        && temp <= temp_humi_alarm.low_temp.unwrap()
                    {
                        temp_alarm = true;
                    } else if temp_humi_alarm.high_temp.is_some()
                        && temp >= temp_humi_alarm.high_temp.unwrap()
                    {
                        temp_alarm = true;
                    }
                    if temp_alarm {
                        if temp_alarm_slient_time.is_none()
                            || (temp_alarm_slient_time.is_some()
                                && temp_alarm_slient_time.unwrap().elapsed()
                                    > Duration::from_secs(30 * 60))
                        {
                            info!("temp alarm trigger!!!");
                            temp_alarm_slient_time = Some(embassy_time::Instant::now());
                            pal::msg_req(Msg::AlarmTempSendReq(temp)).await;
                        }
                    } else {
                        if temp_alarm_slient_time.is_some() {
                            info!("temp alarm recover!!!");
                            temp_alarm_slient_time = None;
                        }
                    }

                    if temp_humi_alarm.low_humi.is_some()
                        && humi <= temp_humi_alarm.low_humi.unwrap()
                    {
                        humi_alarm = true;
                    } else if temp_humi_alarm.high_humi.is_some()
                        && humi >= temp_humi_alarm.high_humi.unwrap()
                    {
                        humi_alarm = true;
                    }
                    if humi_alarm {
                        if humi_alarm_slient_time.is_none()
                            || (humi_alarm_slient_time.is_some()
                                && humi_alarm_slient_time.unwrap().elapsed()
                                    > Duration::from_secs(30 * 60))
                        {
                            info!("humi alarm trigger!!!");
                            humi_alarm_slient_time = Some(embassy_time::Instant::now());
                            pal::msg_req(Msg::AlarmHumiSendReq(humi)).await;
                        }
                    } else {
                        if humi_alarm_slient_time.is_some() {
                            info!("humi alarm recover!!!");
                            humi_alarm_slient_time = None;
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

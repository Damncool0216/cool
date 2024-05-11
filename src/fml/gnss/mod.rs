use core::sync::atomic::{AtomicU8, Ordering};

use super::acc::fml_acc_status_get;
use super::storage::*;
use super::FmlDataProducer;
use super::FmlGnssStatus;
use crate::error;
use crate::fml;
use crate::fml::FmlSystimeUpdateSource;
use crate::{
    info,
    pal::{self, Msg, MsgQueue},
};
use embassy_sync::channel;
use function_name::named;

static FML_GNSS_CTL_QUEUE: MsgQueue<10> = channel::Channel::new();
static FML_GNSS_DATA_QUEUE: MsgQueue<20> = channel::Channel::new();

static FML_GNSS_STATUS: AtomicU8 = AtomicU8::new(0);

pub fn fml_gnss_status_get() -> FmlGnssStatus {
    FML_GNSS_STATUS.load(Ordering::Relaxed).into()
}

#[named]
async fn fml_gnss_status_set(new_status: FmlGnssStatus) {
    if fml_gnss_status_get() == new_status {
        return;
    }
    FML_GNSS_STATUS.store(new_status as u8, Ordering::Relaxed);
    info!("{:?}", new_status);
    match new_status {
        FmlGnssStatus::Off => {}
        FmlGnssStatus::On => {}
        FmlGnssStatus::Fix2D => {}
        FmlGnssStatus::Fix3D => {}
        FmlGnssStatus::NotOpen => {}
    }
}

#[inline]
pub(super) async fn msg_rpy(msg: Msg) {
    match msg {
        Msg::GnssGetLoactionRpy(_) => FML_GNSS_DATA_QUEUE.send(msg).await,
        Msg::ModemReady => FML_GNSS_CTL_QUEUE.send(msg).await,
        _ => FML_GNSS_CTL_QUEUE.send(msg).await,
    }
}

pub async fn fml_gnss_on_req() {
    if fml_gnss_status_get() < FmlGnssStatus::On {
        pal::msg_req(Msg::GnssOpenReq).await
    }
}

pub async fn fml_gnss_off_req() {
    if fml_gnss_status_get() > FmlGnssStatus::Off {
        pal::msg_req(Msg::GnssCloseReq).await
    }
}

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
pub async fn fml_gnss_control_task() {
    loop {
        let msg = FML_GNSS_CTL_QUEUE.receive().await;
        info!("{:?}", msg);
        match msg {
            Msg::GnssOpenRpy(ok) => {
                if ok {
                    fml_gnss_status_set(FmlGnssStatus::On).await;
                    FML_GNSS_DATA_QUEUE.send(Msg::GnssGetLocationReq).await;
                }
            }
            Msg::GnssCloseRpy(ok) => {
                if ok {
                    fml_gnss_status_set(FmlGnssStatus::Off).await
                }
            }
            Msg::ModemReady => {
                if fml_gnss_status_get() > FmlGnssStatus::Off {
                    fml_gnss_status_set(FmlGnssStatus::Off).await;
                    fml_gnss_on_req().await
                }
            }
            _ => {}
        }
    }
}

#[named]
fn fml_gnss_check_error(gnss_data: &FmlGnssRawData) -> i8 {
    let mut ret = 0;
    if gnss_data.fix < 2 {
        ret = -1;
    } else if gnss_data.spkm > 300.0 {
        ret = -2;
    } else if let Some(course) = gnss_data.cog {
        if course > 360.0 || course < 0.0 {
            ret = -3;
        }
    } else if gnss_data.longitude == 0.0 || gnss_data.latitude == 0.0 {
        ret = -4;
    }
    if ret != 0 {
        error!("ret:{}", ret);
    }
    return ret;
}

fn fml_gnss_first_loc_filter(gnss_data: &FmlGnssRawData) -> bool {
    if gnss_data.hdop > 3.5 {
        return false;
    }
    return true;
}

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
pub(super) async fn fml_gnss_data_filter_task(mut producer: FmlDataProducer<'static, FmlGnssData>) {
    let mut cur_gnss = FmlGnssRawData::default();
    let mut last_gnss;
    // let mut first = 1;
    // let mut filter_tick = 0;
    // let mut gps_baud_count = 0;
    let mut gnss_filter_cnt = 0;
    let mut abnormal_cnt = 0;
    loop {
        let mut is_fix = false;
        let msg = FML_GNSS_DATA_QUEUE.receive().await;
        info!("{:?}", msg);
        match msg {
            Msg::GnssGetLoactionRpy(new_gnss) => {
                is_fix = loop {
                    if fml_gnss_status_get() == FmlGnssStatus::Off {
                        gnss_filter_cnt = 0;
                        break false;
                    }
                    if new_gnss.is_none() {
                        info!("new_gnss not fix");
                        gnss_filter_cnt = 0;
                        break false;
                    }
                    last_gnss = cur_gnss.clone();
                    cur_gnss = new_gnss.clone().unwrap();
                    if fml_gnss_check_error(&cur_gnss) != 0 {
                        break false;
                    }

                    if gnss_filter_cnt < 2 && !fml_gnss_first_loc_filter(&cur_gnss) {
                        gnss_filter_cnt = gnss_filter_cnt + 1;
                        break false;
                    }
                    if cur_gnss.spkm > 5.0
                        && cur_gnss.latitude == last_gnss.latitude
                        && cur_gnss.longitude == last_gnss.longitude
                    {
                        abnormal_cnt = abnormal_cnt + 1;
                        error!("abnormal_cnt: {}", abnormal_cnt);
                        break false;
                    }
                    abnormal_cnt = 0;
                    break true;
                };
            }
            Msg::GnssGetLocationReq => {}
            _ => {
                unreachable!();
            }
        }
        info!("is_fix:{}", is_fix);
        if is_fix {
            if cur_gnss.fix == 2 {
                fml_gnss_status_set(FmlGnssStatus::Fix2D).await;
            } else if cur_gnss.fix == 3 {
                fml_gnss_status_set(FmlGnssStatus::Fix3D).await;
            }
            fml::fml_system_time_set(FmlSystimeUpdateSource::Gnss, Some(cur_gnss.utc_stamp)).await;
            let data = FmlGnssData::new(1, cur_gnss.longitude, cur_gnss.latitude);
            producer.enqueue(data).ok();
            info!(
                "gnss enqueue!!! {}/{} at {}",
                producer.len(),
                producer.capacity(),
                fml::fml_system_time_get_ms().await
            );
            super::net::fml_net_mqtt_pub_req().await;
        }

        let gnss_state = fml_gnss_status_get();
        if fml_acc_status_get() == FmlAccStatus::Off
            || gnss_state < FmlGnssStatus::On
            || gnss_state == FmlGnssStatus::NotOpen
        {
            pal::msg_req(Msg::GnssCloseReq).await;
            continue;
        }

        if is_fix {
            embassy_time::Timer::after_secs(30).await;
        } else {
            embassy_time::Timer::after_secs(5).await;
        }
        pal::msg_req(Msg::GnssGetLocationReq).await;
    }
}

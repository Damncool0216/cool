use core::sync::atomic::{AtomicU8, Ordering};

use super::storage::*;
use super::FmlDataProducer;
use super::FmlGnssStatus;
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

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
pub(super) async fn fml_gnss_data_filter_task(mut producer: FmlDataProducer<'static, FmlGnssData>) {
    let mut cur_gnss = None;
    loop {
        let msg = FML_GNSS_DATA_QUEUE.receive().await;
        info!("{:?}", msg);
        match msg {
            Msg::GnssGetLoactionRpy(new_gnss) => {
                if true {
                    cur_gnss = new_gnss;
                }
                if let Some(s) = &cur_gnss {
                    if s.fix == 3 {
                        fml_gnss_status_set(FmlGnssStatus::Fix3D).await;
                    } else {
                        fml_gnss_status_set(FmlGnssStatus::Fix2D).await;
                    }
                    let data = FmlGnssData::new(1, s.longitude, s.latitude);
                    producer.enqueue(data).ok();
                    info!("gnss enqueue!!! {}/{}", producer.len(), producer.capacity());
                    super::net::fml_net_mqtt_pub_req().await;
                }
            }
            _ => {}
        }
        let gnss_state = fml_gnss_status_get();
        if gnss_state < FmlGnssStatus::On || gnss_state == FmlGnssStatus::NotOpen {
            pal::msg_req(Msg::GnssCloseReq).await;
            continue;
        }
        if fml_gnss_status_get() < FmlGnssStatus::Fix2D {
            embassy_time::Timer::after_secs(5).await;
        } else {
            embassy_time::Timer::after_secs(30).await;
        }
        pal::msg_req(Msg::GnssGetLocationReq).await;
    }
}

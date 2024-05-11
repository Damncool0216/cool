use core::sync::atomic::{self, AtomicIsize, AtomicU8};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use function_name::named;
use serde_json_core::heapless::spsc::Queue;
use static_cell::StaticCell;

use self::storage::*;
use crate::{
    info,
    pal::{self, Msg},
};

pub mod acc;
//pub mod alarm;
pub mod gnss;
pub mod net;
pub mod storage;
pub mod temp;

pub type FmlDataQueue<T> = serde_json_core::heapless::spsc::Queue<T, 128>;
pub type FmlDataProducer<'a, T> = serde_json_core::heapless::spsc::Producer<'a, T, 128>;
pub type FmlDataComsumer<'a, T> = serde_json_core::heapless::spsc::Consumer<'a, T, 128>;

pub(super) fn init(spawner: &embassy_executor::Spawner) {
    static T_INIT: StaticCell<FmlDataQueue<FmlTempHumiData>> = StaticCell::new();
    static G_INIT: StaticCell<FmlDataQueue<FmlGnssData>> = StaticCell::new();
    let (temp_p, temp_c) = T_INIT.init(Queue::new()).split();
    let (gnss_p, gnss_c) = G_INIT.init(Queue::new()).split();
    spawner.spawn(storage::fml_storage_task()).unwrap();

    spawner.spawn(temp::fml_temp_msg_rpy_task(temp_p)).unwrap();
    spawner.spawn(temp::fml_temp_detect_task()).unwrap();

    spawner.spawn(net::fml_net_status_task()).unwrap();
    spawner.spawn(net::fml_net_recv_task()).unwrap();
    spawner
        .spawn(net::fml_net_send_task(temp_c, gnss_c))
        .unwrap();

    spawner.spawn(gnss::fml_gnss_control_task()).unwrap();
    spawner
        .spawn(gnss::fml_gnss_data_filter_task(gnss_p))
        .unwrap();

    spawner.spawn(acc::fml_acc_msg_rpy_task()).unwrap();
}

#[inline]
pub(crate) async fn msg_rpy(msg: Msg) {
    match msg {
        msg if msg > Msg::TsensorMsgBegin && msg < Msg::TsensorMsgEnd => temp::msg_rpy(msg).await,
        msg if msg > Msg::GsensorMsgBegin && msg < Msg::GsensorMsgEnd => acc::msg_rpy(msg).await,
        msg if msg > Msg::NetMsgBegin && msg < Msg::NetMsgEnd => net::msg_rpy(msg).await,
        msg if msg > Msg::MqttMsgBegin && msg < Msg::MqttMsgEnd => net::msg_rpy(msg).await,
        msg if msg > Msg::GnssMsgBegin && msg < Msg::GnssMsgEnd => gnss::msg_rpy(msg).await,
        msg if msg == Msg::ModemReady => {
            gnss::msg_rpy(Msg::ModemReady).await;
            net::msg_rpy(Msg::ModemReady).await;
        }
        _ => {}
    }
}
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum FmlSystimeUpdateSource {
    None,
    Gnss,
    Net,
}

static LAST_TIME_SOURCE: AtomicU8 = AtomicU8::new(0);
static LAST_TIME_STAMP: AtomicIsize = AtomicIsize::new(0);
static LAST_RTC_TIEM: AtomicIsize = AtomicIsize::new(0);
static LAST_INSTANT_TIME: Mutex<CriticalSectionRawMutex, Option<embassy_time::Instant>> = Mutex::new(None);
#[named]
//return utc ms time fix with rtc
pub async fn fml_system_time_set(update_source: FmlSystimeUpdateSource, new_utc: Option<i64>) {
    let last_instant = &mut *LAST_INSTANT_TIME.lock().await;

    let mut need_update = false;
    let last_rtc = LAST_RTC_TIEM.load(atomic::Ordering::Relaxed);
    let cur_rtc = pal::rtc::pal_rtc_get_time_ms().await as isize;
    if update_source as u8 >= LAST_TIME_SOURCE.load(atomic::Ordering::Relaxed) {
        if cur_rtc - last_rtc > 4 * 60 * 1000
            || update_source as u8 > LAST_TIME_SOURCE.load(atomic::Ordering::Relaxed)
        {
            need_update = true;
        }
    }
    info!("{:?},{}", update_source, need_update);
    if need_update {
        match update_source {
            FmlSystimeUpdateSource::Net => {
                if let Some(utc) = new_utc {
                    LAST_TIME_SOURCE
                        .store(FmlSystimeUpdateSource::Net as u8, atomic::Ordering::Relaxed);
                    LAST_TIME_STAMP.store(utc as isize, atomic::Ordering::Relaxed);
                    LAST_RTC_TIEM.store(
                        pal::rtc::pal_rtc_get_time_ms().await as isize,
                        atomic::Ordering::Relaxed,
                    );
                    *last_instant = Some(embassy_time::Instant::now());
                }
            }
            FmlSystimeUpdateSource::Gnss => {
                if let Some(utc) = new_utc {
                    LAST_TIME_SOURCE.store(
                        FmlSystimeUpdateSource::Gnss as u8,
                        atomic::Ordering::Relaxed,
                    );
                    LAST_TIME_STAMP.store(utc as isize, atomic::Ordering::Relaxed);
                    LAST_RTC_TIEM.store(
                        pal::rtc::pal_rtc_get_time_ms().await as isize,
                        atomic::Ordering::Relaxed,
                    );
                    *last_instant = Some(embassy_time::Instant::now());
                }
            }
            _ => {}
        }
    }
}

#[named]
pub async fn fml_system_time_get_ms() -> i64 {
    let last_source = LAST_TIME_SOURCE.load(atomic::Ordering::Relaxed);
    if last_source == FmlSystimeUpdateSource::None as u8 {
        //not init yet
        return 0;
    }

    let last_utc = LAST_TIME_STAMP.load(atomic::Ordering::Relaxed) as u64;
    let last_rtc = LAST_RTC_TIEM.load(atomic::Ordering::Relaxed) as u64;

    let cur_rtc = pal::rtc::pal_rtc_get_time_ms().await;
    let cur_utc_ms_from_rtc = last_utc * 1000 + cur_rtc - last_rtc;
    let mut cur_utc_ms_from_instant = 0;
    {
        let last_instant = &*LAST_INSTANT_TIME.lock().await;
        if let Some(l) = last_instant {
            cur_utc_ms_from_instant = last_utc * 1000 + l.elapsed().as_millis();
        }
    }

    info!(
        "source: {}, last_utc: {}, last_rtc: {}, cur_rtc: {}, cur_utc_ms_from_rtc: {}, cur_utc_from_instant: {}, cur_utc_ms_from_instant",
        last_source, last_utc, last_rtc, cur_rtc, cur_utc_ms_from_rtc, cur_utc_ms_from_instant
    );
    
    return cur_utc_ms_from_instant as i64;
    //return cur_utc_ms_from_rtc as i64; // shit
}

pub async fn fml_system_time_get_sec() -> i64 {
    let ms = fml_system_time_get_ms().await;
    if ms == 0 {
        return 0;
    }
    return ms / 1000;
}

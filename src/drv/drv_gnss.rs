use ec800m_at::general::types::OnOff;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::{self, Channel}};
use log::info;
use nmea::{Nmea, SentenceType};
use static_cell::StaticCell;

use crate::drv::drv_at::drv_at_client_handle_get;

pub enum DrvGnssMsg {
    None,
    Open,
}
type DrvGnssTaskQueueType = Channel<CriticalSectionRawMutex, DrvGnssMsg, 20>;
static DRV_GNSS_TASK_QUEUE: DrvGnssTaskQueueType = channel::Channel::new();
pub fn drv_gnss_get_task_queue() -> &'static DrvGnssTaskQueueType {
    return &DRV_GNSS_TASK_QUEUE;
}

#[embassy_executor::task]
pub async fn drv_gnss_task() {
    static NMEAPARSER: StaticCell<Nmea> = StaticCell::new();
    let _nmea_parser = NMEAPARSER.init(
        Nmea::create_for_navigation(&[
            SentenceType::RMC,
            SentenceType::GSV,
            SentenceType::GGA,
            SentenceType::GNS,
            SentenceType::GLL,
        ])
        .unwrap(),
    );
    loop {
        match drv_gnss_get_task_queue().receive().await {
            DrvGnssMsg::None => {
                info!("DrvGnssMsg::None")
            }
            DrvGnssMsg::Open => {
                if let Some(client) = drv_at_client_handle_get().lock().await.as_mut() {
                    info!("DrvGnssMsg::Open");
                    if let Ok(_) = client.gps_set_sw(OnOff::On).await {
                        info!("gps open ok");
                    } else {
                        client.gps_set_sw(OnOff::Off).await.ok();
                    }
                }
            }
        }
        // {
        //     match state {
        //         GnssTaskState::Init => {
        //             if let Some(client) = ATCLIENT.lock().await.as_mut() {
        //                 client.gpscfg_set_nmea_src(OnOff::On).await.ok();
        //                 client.gpscfg_set_outport(Outport::UartDebug).await.ok();
        //                 client.gpscfg_set_nmea_src(OnOff::On).await.ok();
        //                 client.gpscfg_set_auto_gps(OnOff::Off).await.ok();
        //                 client.gpscfg_set_ap_flash(OnOff::On).await.ok();
        //                 state = GnssTaskState::Close;
        //             }
        //         }
        //         GnssTaskState::Close => {}
        //         _ => {}
        //     }
        // }
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}
use ec800m_at::{
    general::types::OnOff,
    gnss::types::{NmeaType, Outport},
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{self, Channel},
};
use log::info;
use nmea::{Nmea, SentenceType};
use static_cell::StaticCell;

use crate::drv::drv_at::drv_at_client_handle_get;

pub enum DrvGnssMsg {
    Init,
    Open,
    Close,
    GetLocation,
}
type DrvGnssTaskQueueType = Channel<CriticalSectionRawMutex, DrvGnssMsg, 20>;
static DRV_GNSS_TASK_QUEUE: DrvGnssTaskQueueType = channel::Channel::new();
pub fn drv_gnss_get_task_queue() -> &'static DrvGnssTaskQueueType {
    return &DRV_GNSS_TASK_QUEUE;
}

#[embassy_executor::task]
pub async fn drv_gnss_task() {
    static NMEAPARSER: StaticCell<Nmea> = StaticCell::new();
    let nmea_parser = NMEAPARSER.init(
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
            DrvGnssMsg::Init => {
                info!("DrvGnssMsg::None")
            }
            DrvGnssMsg::Open => {
                if let Some(client) = drv_at_client_handle_get().lock().await.as_mut() {
                    info!("DrvGnssMsg::Open");
                    client.gpscfg_set_outport(Outport::UartDebug).await.ok();
                    client.gpscfg_set_nmea_src(OnOff::On).await.ok();
                    client.gpscfg_set_auto_gps(OnOff::Off).await.ok();
                    client.gpscfg_set_ap_flash(OnOff::On).await.ok();
                    match client.gps_set_sw(OnOff::On).await {
                        Ok(_) => {
                            info!("gps open ok");
                        }
                        Err(e) => {
                            info!("gps open err {:?}", e);
                        }
                    }
                }
            }
            DrvGnssMsg::Close => {
                if let Some(client) = drv_at_client_handle_get().lock().await.as_mut() {
                    info!("DrvGnssMsg::Close");
                    if let Ok(_) = client.gps_set_sw(OnOff::Off).await {
                        info!("gps close ok");
                    }
                }
            }
            DrvGnssMsg::GetLocation => {
                if let Some(client) = drv_at_client_handle_get().lock().await.as_mut() {
                    info!("DrvGnssMsg::GetLocation");
                    if let Ok(gsvs) = client.gps_get_nmea(NmeaType::GSV).await {
                        for gsv in gsvs {
                            nmea_parser.parse(&gsv).ok();
                        }
                    }
                    if let Ok(gsas) = client.gps_get_nmea(NmeaType::GSA).await {
                        for gsa in gsas {
                            nmea_parser.parse(&gsa).ok();
                        }
                    }
                    if let Ok(ggas) = client.gps_get_nmea(NmeaType::GGA).await {
                        for gga in ggas {
                            nmea_parser.parse(&gga).ok();
                        }
                    }
                    if let Ok(rmcs) = client.gps_get_nmea(NmeaType::RMC).await {
                        for rmc in rmcs {
                            nmea_parser.parse(&rmc).ok();
                        }
                    }
                    info!(
                        "time:{:?} latitude:{:?} longitude:{:?}, fix_used:{:?}, hdop:{:?}, vdop:{:?}, pdop:{:?}, prns:{:?}",
                        nmea_parser.fix_time,
                        nmea_parser.latitude,
                        nmea_parser.longitude,
                        nmea_parser.num_of_fix_satellites,
                        nmea_parser.hdop,
                        nmea_parser.vdop,
                        nmea_parser.pdop,
                        nmea_parser.fix_satellites_prns
                    );
                }
            }
        }
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}

pub trait GnssHandle {
    async fn init();
    async fn open();
    async fn close();
    async fn get_location();
}
pub struct DrvGnssHandle;
impl GnssHandle for DrvGnssHandle {
    async fn init() {
        drv_gnss_get_task_queue().send(DrvGnssMsg::Init).await
    }
    async fn open() {
        drv_gnss_get_task_queue().send(DrvGnssMsg::Open).await
    }
    async fn close() {
        drv_gnss_get_task_queue().send(DrvGnssMsg::Close).await
    }
    async fn get_location() {
        drv_gnss_get_task_queue().send(DrvGnssMsg::GetLocation).await
    }
}

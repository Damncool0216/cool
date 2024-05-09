use embassy_sync::channel;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use serde::{Deserialize, Serialize};
use serde_json_core::heapless::String;

use crate::pal::MsgQueue;
///net attach status
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum FmlNetAttachStatus {
    NoSim = 0,
    SimReady,
    NoService,
    InCall,
    ///2G
    GSMService,
    ///4G
    LTEService,
}
impl From<u8> for FmlNetAttachStatus {
    fn from(value: u8) -> Self {
        match value {
            v if v == FmlNetAttachStatus::NoSim as u8 => FmlNetAttachStatus::NoSim,
            v if v == FmlNetAttachStatus::SimReady as u8 => FmlNetAttachStatus::SimReady,
            v if v == FmlNetAttachStatus::NoService as u8 => FmlNetAttachStatus::NoService,
            v if v == FmlNetAttachStatus::InCall as u8 => FmlNetAttachStatus::InCall,
            v if v == FmlNetAttachStatus::GSMService as u8 => FmlNetAttachStatus::GSMService,
            v if v == FmlNetAttachStatus::LTEService as u8 => FmlNetAttachStatus::LTEService,
            _ => FmlNetAttachStatus::NoSim,
        }
    }
}

///platform conn status
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum FmlNetConnStatus {
    Down = 0,
    Connecting,
    Connected,
    Logined,
    NotConnect,
}

impl From<u8> for FmlNetConnStatus {
    fn from(value: u8) -> Self {
        match value {
            v if v == FmlNetConnStatus::Down as u8 => FmlNetConnStatus::Down,
            v if v == FmlNetConnStatus::Connecting as u8 => FmlNetConnStatus::Connecting,
            v if v == FmlNetConnStatus::Connected as u8 => FmlNetConnStatus::Connected,
            v if v == FmlNetConnStatus::Logined as u8 => FmlNetConnStatus::Logined,
            v if v == FmlNetConnStatus::NotConnect as u8 => FmlNetConnStatus::NotConnect,
            _ => FmlNetConnStatus::Down,
        }
    }
}

pub struct FmLNetNvm {
    pub mqtt_version: u8,
    pub mqtt_idx: u8,
    pub mqtt_server: (String<50>, u16),
    pub mqtt_client_id: String<50>,
    pub mqtt_usename: String<50>,
    pub mqtt_password: String<256>,
    pub dp_topic: String<50>,
    pub send_type: Option<FmlNetSendType>,
}

impl FmLNetNvm {
    pub fn default() -> Self {
        let mqtt_client_id = String::try_from(env!("MQTT_CLIENT_ID")).unwrap();
        let mqtt_usename = String::try_from(env!("MQTT_USERNAME")).unwrap();
        let mut dp_topic = String::new();
        dp_topic.push_str("$sys/").unwrap();
        dp_topic.push_str(mqtt_usename.as_str()).unwrap();
        dp_topic.push('/').unwrap();
        dp_topic.push_str(mqtt_client_id.as_str()).unwrap();
        dp_topic.push_str("/dp/post/json").unwrap();
        Self {
            mqtt_idx: 1,
            mqtt_version: 1,
            mqtt_server: (
                String::try_from(env!("MQTT_SERVER_DOMAIN")).unwrap(),
                env!("MQTT_SERVER_PORT").parse().unwrap(),
            ),
            mqtt_client_id,
            mqtt_usename,
            mqtt_password: String::try_from(env!("MQTT_PASSWORD")).unwrap(),
            dp_topic,
            send_type: None,
        }
    }
}

/*
{
    "id": 0,
    "dp": {
        "temp": [{
            "v": 24.673
        }],
        "humi": [{
            "v":81.855
        }]
}   }
*/
#[derive(Serialize, Clone, Debug, Deserialize)]
pub struct FmlTempHumiData {
    id: u16,
    dp: FmlTempHumiDataPoint,
}

#[derive(Serialize, Clone, Debug, Deserialize)]
struct FmlTempHumiDataPoint {
    pub temp: [FmlTempHumiKV; 1],
    pub humi: [FmlTempHumiKV; 1],
}

#[derive(Serialize, Clone, Debug, Deserialize)]
struct FmlTempHumiKV {
    v: f32,
}

impl FmlTempHumiData {
    pub fn new(id: u16, temp: f32, humi: f32) -> Self {
        FmlTempHumiData {
            id,
            dp: FmlTempHumiDataPoint {
                temp: [FmlTempHumiKV { v: temp }],
                humi: [FmlTempHumiKV { v: humi }],
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FmLTempNvm {
    /// temp humi detect interval min
    pub detect_inv: u32,
}

impl FmLTempNvm {
    pub fn default() -> Self {
        Self { detect_inv: 5 }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
#[repr(u8)]
pub enum FmlAccStatus {
    Off = 0,
    On = 1,
}
impl From<u8> for FmlAccStatus {
    fn from(value: u8) -> Self {
        match value {
            x if x == FmlAccStatus::Off as u8 => FmlAccStatus::Off,
            x if x == FmlAccStatus::On as u8 => FmlAccStatus::On,
            _ => FmlAccStatus::Off,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum FmlGnssStatus {
    Off = 0,
    On,
    Fix2D,
    Fix3D,
    NotOpen,
}
impl From<u8> for FmlGnssStatus {
    fn from(value: u8) -> Self {
        match value {
            x if x == Self::Off as u8 => Self::Off,
            x if x == Self::On as u8 => Self::On,
            x if x == Self::Fix2D as u8 => Self::Fix2D,
            x if x == Self::Fix3D as u8 => Self::Fix3D,
            x if x == Self::NotOpen as u8 => Self::NotOpen,
            _ => Self::Off,
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct FmlGnssRawData {
    pub latitude: f32,
    pub longitude: f32,
    pub hdop: f32,
    pub altitude: f32,
    pub fix: u8,
    pub cog: Option<f32>,
    pub spkm: f32,
    pub spkn: f32,
    pub nsat: u8,
    //ub utc_stamp: Option<f32>,
}

#[derive(Serialize, Clone, Debug, Deserialize)]
struct FmlGnssV {
    pub lat: f32,
    pub lon: f32,
}

#[derive(Serialize, Clone, Debug, Deserialize)]
struct FmlGnssKV {
    v: FmlGnssV,
}

#[derive(Serialize, Clone, Debug, Deserialize)]
struct FmlGnssDataPoint {
    pub location: [FmlGnssKV; 1],
}

#[derive(Serialize, Clone, Debug, Deserialize)]
pub struct FmlGnssData {
    id: u16,
    dp: FmlGnssDataPoint,
}

impl FmlGnssData {
    pub fn new(id: u16, lon: f32, lat: f32) -> Self {
        FmlGnssData {
            id,
            dp: FmlGnssDataPoint {
                location: [FmlGnssKV {
                    v: FmlGnssV { lat, lon },
                }],
            },
        }
    }
}

pub struct FmlGnssNvm {
    pub gps_sw: bool,
    pub acc_on_itv_sec: u16,  //运动下上报间隔
    pub acc_off_itv_sec: u16, //静止下上报间隔
    pub last_gnss_data: Option<FmlGnssRawData>,
    pub cur_gnss_data: Option<FmlGnssRawData>,
}

impl FmlGnssNvm {
    pub fn default() -> Self {
        Self {
            gps_sw: true,
            acc_on_itv_sec: 60,
            acc_off_itv_sec: 0,
            last_gnss_data: None,
            cur_gnss_data: None,
        }
    }
}

// store fml config
pub static FML_TEMP_NVM: Mutex<CriticalSectionRawMutex, Option<FmLTempNvm>> = Mutex::new(None);
pub static FML_NET_NVM: Mutex<CriticalSectionRawMutex, Option<FmLNetNvm>> = Mutex::new(None);
pub static FML_GNSS_NVM: Mutex<CriticalSectionRawMutex, Option<FmlGnssNvm>> = Mutex::new(None);

#[derive(Clone, Debug)]
pub enum FmlNetSendType {
    Location(FmlGnssData),
    TempHumi(FmlTempHumiData),
}

static FML_STORAGE_DATA_QUEUE: MsgQueue<20> = channel::Channel::new();
#[embassy_executor::task]
pub async fn fml_storage_task() {
    {
        *(FML_TEMP_NVM.lock().await) = Some(FmLTempNvm::default());
        *(FML_NET_NVM.lock().await) = Some(FmLNetNvm::default());
        *(FML_GNSS_NVM.lock().await) = Some(FmlGnssNvm::default());
    }
    loop {
        let msg = FML_STORAGE_DATA_QUEUE.receive().await;
        match msg {
            _ => {}
        }
    }
}

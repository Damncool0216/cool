use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use function_name::named;
use heapless::String;

use crate::mdebug;

#[derive(Debug, Clone)]
struct FmLNetNvm {
    mqtt_server: (String<50>, u16),
    mqtt_client_id: String<50>,
    mqtt_usename: String<50>,
    mqtt_password: String<256>,
}

///net attach status
enum FmlNetAttachStatus {
    NoService,
    InCall,
    ///2G
    GSMService,
    ///4G
    LTEService,
}

///platform conn status
enum FmlNetConnStatus {
    Down,
    Pending,
    Connecting,
    Connected,
    Logined,
    NotConnect,
}

impl FmLNetNvm {
    pub fn default() -> Self {
        Self {
            mqtt_server: (
                String::try_from(env!("MQTT_SERVER_DOMAIN")).unwrap(),
                env!("MQTT_SERVER_PORT").parse().unwrap(),
            ),
            mqtt_client_id: String::try_from(env!("MQTT_CLIENT_ID")).unwrap(),
            mqtt_usename: String::try_from(env!("MQTT_USERNAME")).unwrap(),
            mqtt_password: String::try_from(env!("MQTT_PASSWORD")).unwrap(),
        }
    }
}

static FML_NET_NVM: Mutex<CriticalSectionRawMutex, Option<FmLNetNvm>> = Mutex::new(None);

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
/// handle net attach
pub(super) async fn fml_net_attach_task() {
    {
        *(FML_NET_NVM.lock().await) = Some(FmLNetNvm::default())
    }
    loop {
        if let Some(fml_net_nvm) = &*FML_NET_NVM.lock().await {
            mdebug!("{:?}", fml_net_nvm);
        }
        embassy_time::Timer::after_secs(60).await;
    }
}

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
/// handle platform connect
pub(super) async fn fml_net_connect_task() {
    loop {
        embassy_time::Timer::after_secs(60).await;
    }
}

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
/// handle net data send
pub(super) async fn fml_net_send_task() {
    loop {
        embassy_time::Timer::after_secs(60).await;
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

pub(super) fn init(spawner: &embassy_executor::Spawner) {}

use atat::{asynch::Client, AtatIngress, Ingress, ResponseSlot, UrcChannel};
use ec800m_at::{client::asynch::Ec800mClient, digester::Ec800mDigester, general::types::OnOff, urc::URCMessages};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{self, Channel},
    mutex::Mutex,
};
use hal::{peripherals::UART1, UartRx, UartTx};
use log::info;
use static_cell::StaticCell;

const RX_SIZE: usize = 1024;
const INGRESS_BUF_SIZE: usize = RX_SIZE;
const URC_SUBSCRIBERS: usize = 0;
const URC_CAPACITY: usize = RX_SIZE * 3;

type AtIngress<'a> =
    Ingress<'a, Ec800mDigester, URCMessages, INGRESS_BUF_SIZE, URC_CAPACITY, URC_SUBSCRIBERS>;

type AtReader<'a> = UartRx<'a, UART1>;
type AtWriter<'a> = UartTx<'a, UART1>;

type AtClientType<'a> = Ec800mClient<'a, AtWriter<'a>, INGRESS_BUF_SIZE>;
type AtClient<'a> = Mutex<CriticalSectionRawMutex, Option<AtClientType<'a>>>;

static AT_CLIENT: AtClient<'static> = Mutex::new(None);
static RES_SLOT: ResponseSlot<INGRESS_BUF_SIZE> = ResponseSlot::new();

#[allow(unused)]
pub enum DrvAtClientMsg {
    None,
}
type DrvAtClientTaskQueueType = Channel<CriticalSectionRawMutex, DrvAtClientMsg, 20>;

static DRV_AT_CLIENT_TASK_QUEUE: DrvAtClientTaskQueueType = channel::Channel::new();

pub fn drv_at_client_get_task_queue() -> &'static DrvAtClientTaskQueueType {
    return &DRV_AT_CLIENT_TASK_QUEUE;
}

pub fn drv_at_client_handle_get() -> &'static AtClient<'static> {
    &AT_CLIENT
}
#[embassy_executor::task()]
pub async fn drv_at_client_task(writer: AtWriter<'static>) {
    static BUF: StaticCell<[u8; 1024]> = StaticCell::new();
    let client = Client::new(
        writer,
        &RES_SLOT,
        BUF.init([0; 1024]),
        atat::Config::default(),
    );
    let mut ec800m_at_client: AtClientType = Ec800mClient::new(client).await.unwrap();
    if let Ok(_) = ec800m_at_client.verify_com_is_working().await {
        info!("At client init ok!");
    }
    ec800m_at_client.at_echo_set(OnOff::Off).await.ok();
    ec800m_at_client.at_config_save().await.ok();
    {
        *(AT_CLIENT.lock().await) = Some(ec800m_at_client);
    }

    loop {
        match drv_at_client_get_task_queue().receive().await {
            DrvAtClientMsg::None => {
                info!("{} DrvAtClientMsg::None", module_path!());
            }
        }
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
pub async fn drv_at_ingress_task(mut reader: AtReader<'static>) {
    static INGRESS_BUF: StaticCell<[u8; INGRESS_BUF_SIZE]> = StaticCell::new();
    static URC_CHANNEL: UrcChannel<URCMessages, URC_CAPACITY, URC_SUBSCRIBERS> = UrcChannel::new();
    let mut ingress: AtIngress<'static> = Ingress::new(
        Ec800mDigester::default(),
        INGRESS_BUF.init([0; INGRESS_BUF_SIZE]),
        &RES_SLOT,
        &URC_CHANNEL,
    );

    ingress.read_from(&mut reader).await
}

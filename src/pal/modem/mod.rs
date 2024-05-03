use atat::{AtatIngress, Ingress, ResponseSlot, UrcChannel};
use function_name::named;
use core::any::Any;
use ec800m_at::{
    client::asynch::Ec800mClient, digester::Ec800mDigester, general::types::OnOff, urc::URCMessages,
};
use embassy_sync::channel;
use hal::{
    peripherals::UART1,
    uart::{UartRx, UartTx},
    Async,
};
use log::info;
use static_cell::StaticCell;

use crate::pal;

use super::{PalMsg, PalQueue};

mod ec800m_at;

const RX_SIZE: usize = 2048;
const INGRESS_BUF_SIZE: usize = RX_SIZE;
const URC_SUBSCRIBERS: usize = 0;
const URC_CAPACITY: usize = RX_SIZE * 3;

type AtReader<'a> = UartRx<'a, UART1, Async>;
type AtWriter<'a> = UartTx<'a, UART1, Async>;

type AtDigester = Ec800mDigester;
type AtClient<'a> = Ec800mClient<'a, AtWriter<'a>, INGRESS_BUF_SIZE>;

static RES_SLOT: ResponseSlot<INGRESS_BUF_SIZE> = ResponseSlot::new();

static PAL_MODEM_TASK_QUEUE: PalQueue<20> = channel::Channel::new();

#[inline]
pub(crate) async fn msg_req(msg: PalMsg) {
    PAL_MODEM_TASK_QUEUE.send(msg).await
}

#[inline]
pub(crate) async fn msg_rpy(msg: PalMsg) {
    pal::msg_rpy(msg).await
}

#[embassy_executor::task()]
#[named]
pub async fn client_task(writer: AtWriter<'static>) {
    static BUF: StaticCell<[u8; 1024]> = StaticCell::new();
    let client = atat::asynch::Client::new(
        writer,
        &RES_SLOT,
        BUF.init([0; 1024]),
        atat::Config::default(),
    );
    let mut at_client = AtClient::new(client).await.unwrap();

    while let Err(_) = at_client.verify_com_is_working().await {
        info!("[{}] At baudrate adapting!", function_name!());
    }
    info!("At client init ok!");
    at_client.at_echo_set(OnOff::Off).await.ok();
    at_client.at_config_save().await.ok();
    at_client.gps_set_sw(OnOff::Off).await.ok();

    loop {
        let msg = PAL_MODEM_TASK_QUEUE.receive().await;
        info!("{:?}", msg.type_id());
        match msg {
            PalMsg::GnssOpenReq => {
                msg_rpy(PalMsg::GnssOpenRpy(
                    at_client.gps_set_sw(OnOff::On).await.is_ok(),
                ))
                .await
            }
            PalMsg::GnssCloseReq => {
                msg_rpy(PalMsg::GnssCloseRpy(
                    at_client.gps_set_sw(OnOff::On).await.is_ok(),
                ))
                .await
            }
            PalMsg::GnssGetLocationReq => {
                at_client.gps_get_location().await.ok();
            }
            _ => {}
        }
        //embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
pub async fn ingress_task(mut reader: AtReader<'static>) {
    static INGRESS_BUF: StaticCell<[u8; INGRESS_BUF_SIZE]> = StaticCell::new();
    static URC_CHANNEL: UrcChannel<URCMessages, URC_CAPACITY, URC_SUBSCRIBERS> = UrcChannel::new();
    let mut ingress = Ingress::new(
        AtDigester::default(),
        INGRESS_BUF.init([0; INGRESS_BUF_SIZE]),
        &RES_SLOT,
        &URC_CHANNEL,
    );

    ingress.read_from(&mut reader).await
}

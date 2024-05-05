use atat::{AtatIngress, Ingress, ResponseSlot, UrcChannel};
use ec800m_at::{
    client::asynch::Ec800mClient, digester::Ec800mDigester, general::types::OnOff, urc::URCMessages,
};
use embassy_sync::channel;
use function_name::named;
use hal::{
    peripherals::UART1,
    uart::{UartRx, UartTx},
    Async,
};

use static_cell::StaticCell;

use crate::{mdebug, minfo, pal};

use super::{Msg, MsgQueue};

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

static PAL_MODEM_TASK_QUEUE: MsgQueue<30> = channel::Channel::new();

#[inline]
pub(super) async fn msg_req(msg: Msg) {
    PAL_MODEM_TASK_QUEUE.send(msg).await
}

#[inline]
pub(super) async fn msg_rpy(msg: Msg) {
    pal::msg_rpy(msg).await
}

#[embassy_executor::task()]
#[allow(unused_macros)]
#[named]
pub(super) async fn pal_at_client_task(writer: AtWriter<'static>) {
    static BUF: StaticCell<[u8; 1024]> = StaticCell::new();
    let client = atat::asynch::Client::new(
        writer,
        &RES_SLOT,
        BUF.init([0; 1024]),
        atat::Config::default(),
    );
    let mut at_client = AtClient::new(client).await.unwrap();

    while let Err(_) = at_client.verify_com_is_working().await {
        mdebug!("At baudrate adapting!");
    }
    mdebug!("At client init ok!");
    at_client.at_echo_set(OnOff::Off).await.ok();
    at_client.at_config_save().await.ok();
    at_client.gps_set_sw(OnOff::Off).await.ok();

    loop {
        let msg = PAL_MODEM_TASK_QUEUE.receive().await;
        minfo!("{:?}", msg);
        match msg {
            Msg::GnssOpenReq => {
                msg_rpy(Msg::GnssOpenRpy(
                    at_client.gps_set_sw(OnOff::On).await.is_ok(),
                ))
                .await
            }
            Msg::GnssCloseReq => {
                msg_rpy(Msg::GnssCloseRpy(
                    at_client.gps_set_sw(OnOff::On).await.is_ok(),
                ))
                .await
            }
            Msg::GnssGetLocationReq => {
                at_client.gps_get_location().await.ok();
            }
            _ => {}
        }
        //embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
pub(super) async fn pal_at_ingress_task(mut reader: AtReader<'static>) {
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

//AT+QMTCFG="version",1,4
//AT+QMTOPEN=1,"mqtts.heclouds.com",1883
//AT+QMTCONN=1,cool,g7R22epB27,version=2018-10-31&res=products%2Fg7R22epB27%2Fdevices%2Fcool&et=1813073329&method=md5&sign=CY0EPrmjfaqD3yiLUn731w%3D%3D

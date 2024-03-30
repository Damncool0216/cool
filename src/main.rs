#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::cell::RefCell;

use esp_backtrace as _;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_hal_async::delay::{self, DelayNs};
use hal::{
    clock::ClockControl,
    embassy, interrupt,
    peripherals::{Peripherals, UART1},
    prelude::*,
    systimer::SystemTimer,
    timer::TimerGroup,
    uart::{self, config::AtCmdConfig, TxRxPins, Uart, UartRx, UartTx},
    Delay, Rng, IO,
};

use log::{error, info};

use nmea::{Nmea, SentenceType};
use static_cell::StaticCell;

use atat::{asynch::Client, helpers::LossyStr, Config, Ingress, UrcChannel};
use atat::{AtatIngress, ResponseSlot};

use ec800m_at::client::asynch::Ec800mClient;
use ec800m_at::digester::Ec800mDigester;
use ec800m_at::general::types::OnOff;
use ec800m_at::urc::URCMessages;
use ec800m_at::{
    client,
    gnss::types::{DeleteType, GnssConfig, NmeaConfig, NmeaType, Outport},
};

// Chunk size in bytes when receiving data. Value should be matched to buffer
// size of receive() calls.
const RX_SIZE: usize = 1044;
// Constants derived from TX_SIZE and RX_SIZE
const INGRESS_BUF_SIZE: usize = RX_SIZE;
const URC_SUBSCRIBERS: usize = 0;
const URC_CAPACITY: usize = RX_SIZE * 3;

macro_rules! singleton {
    ($val:expr) => {{
        type T = impl Sized;
        static STATIC_CELL: StaticCell<T> = StaticCell::new();
        let (x,) = STATIC_CELL.init(($val,));
        x
    }};
}

type AtIngress<'a> =
    Ingress<'a, Ec800mDigester, URCMessages, INGRESS_BUF_SIZE, URC_CAPACITY, URC_SUBSCRIBERS>;
type AtReader<'a> = UartRx<'a, UART1>;

type AtWriter<'a> = UartTx<'a, UART1>;

type AtClient<'a> = Ec800mClient<'a, AtWriter<'a>, INGRESS_BUF_SIZE>;
#[main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timer_group0);

    let config = uart::config::Config {
        baudrate: 115200,
        data_bits: uart::config::DataBits::DataBits8,
        parity: uart::config::Parity::ParityNone,
        stop_bits: uart::config::StopBits::STOP1,
    };

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let pins = TxRxPins::new_tx_rx(
        io.pins.gpio0.into_push_pull_output(),
        io.pins.gpio1.into_floating_input(),
    );

    let mut serial1 = Uart::new_with_config(peripherals.UART1, config, Some(pins), &clocks);

    serial1.set_rx_fifo_full_threshold(6).unwrap();
    serial1.set_at_cmd(AtCmdConfig::new(None, None, None, b'\r', None));
    serial1.listen_at_cmd();
    serial1.listen_rx_fifo_full();
    let (writer, reader) = serial1.split();

    static INGRESS_BUF: StaticCell<[u8; INGRESS_BUF_SIZE]> = StaticCell::new();
    static RES_SLOT: ResponseSlot<INGRESS_BUF_SIZE> = ResponseSlot::new();
    static URC_CHANNEL: UrcChannel<URCMessages, URC_CAPACITY, URC_SUBSCRIBERS> = UrcChannel::new();

    let ingress: AtIngress<'static> = Ingress::new(
        Ec800mDigester::default(),
        INGRESS_BUF.init([0; INGRESS_BUF_SIZE]),
        &RES_SLOT,
        &URC_CHANNEL,
    );

    static BUF: StaticCell<[u8; 1024]> = StaticCell::new();
    let client = Client::new(writer, &RES_SLOT, BUF.init([0; 1024]), Config::default());
    let ec800m_at_client: AtClient = Ec800mClient::new(client).await.unwrap();
    let nmea_parser = nmea::Nmea::create_for_navigation(&[SentenceType::RMC]).unwrap();
    //let mut parser = NmeaParser::new();
    spawner.spawn(ingress_task(ingress, reader)).ok();
    spawner.spawn(at_client_task(ec800m_at_client)).ok();
    //spawner.spawn(gnss_task(nmea_parser)).ok();
    spawner.spawn(i2c_master_task()).ok();
}

#[embassy_executor::task]
async fn ingress_task(mut ingress: AtIngress<'static>, mut reader: AtReader<'static>) {
    ingress.read_from(&mut reader).await
}

#[embassy_executor::task(pool_size = 4)]
async fn at_client_task(mut client: AtClient<'static>) -> ! {
    let mut state = 1;
    loop {
        info!("state:{}", state);
        match state {
            1 => {
                if let Ok(_) = client.verify_com_is_working().await {
                    info!("com ok");
                    state = state + 1;
                }
            }
            2 => {
                if let Ok(_) = client.at_echo_set(OnOff::Off).await {
                    info!("ATE0 ok");
                    state = state + 1;
                }
                client.at_config_save().await.ok();
            }
            3 => {
                state = state + 1;
            }
            4 => {
                client.gpscfg_set_outport(Outport::UartDebug).await.ok();
                client.gpscfg_set_nmea_src(OnOff::On).await.ok();
                client.gpscfg_set_auto_gps(OnOff::On).await.ok();
                client.gpscfg_set_ap_flash(OnOff::On).await.ok();
                if let Ok(_) = client.gps_set_sw(OnOff::On).await {
                    info!("gps open ok");
                    client.gps_set_del(DeleteType::NotDel).await.ok();
                    state += 1;
                } else {
                    client.gps_set_sw(OnOff::Off).await.ok();
                }
            }
            5 => {
                client.gpscfg_set_nmea_src(OnOff::On).await.ok();

                if let Ok(s) = client.gps_get_nmea(NmeaType::GSV).await {
                    let mut nmea_parser = nmea::Nmea::create_for_navigation(&[
                        SentenceType::RMC,
                        SentenceType::GGA,
                        SentenceType::GSV,
                        SentenceType::GNS,
                    ])
                    .unwrap();
                    for nmea_sentence in s {
                        info!("parsing nmea {}", nmea_sentence);
                        nmea_parser.parse(nmea_sentence.as_str()).ok();
                        info!("{:?} {:?}", nmea_parser.longitude, nmea_parser.latitude);
                    }
                }
                if let Ok(s) = client.gps_get_nmea(NmeaType::GGA).await {
                    let mut nmea_parser = nmea::Nmea::create_for_navigation(&[
                        SentenceType::RMC,
                        SentenceType::GGA,
                        SentenceType::GSV,
                        SentenceType::GNS,
                    ])
                    .unwrap();
                    for nmea_sentence in s {
                        info!("parsing nmea {}", nmea_sentence);
                        nmea_parser.parse(nmea_sentence.as_str()).ok();
                        info!("{:?} {:?}", nmea_parser.longitude, nmea_parser.latitude);
                    }
                }
                if let Ok(s) = client.gps_get_nmea(NmeaType::RMC).await {
                    let mut nmea_parser = nmea::Nmea::create_for_navigation(&[
                        SentenceType::RMC,
                        SentenceType::GGA,
                        SentenceType::GSV,
                        SentenceType::GNS,
                    ])
                    .unwrap();
                    for nmea_sentence in s {
                        info!("parsing nmea {}", nmea_sentence);
                        nmea_parser.parse(nmea_sentence.as_str()).ok();
                        info!("{:?} {:?}", nmea_parser.longitude, nmea_parser.latitude);
                    }
                }
            }
            _ => {
                state = 0;
            }
        }
        embassy_time::Timer::after(embassy_time::Duration::from_secs(2)).await;
    }
}

#[embassy_executor::task]
async fn gnss_task() {
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(2)).await;
    }
}

#[embassy_executor::task]
async fn i2c_master_task() {
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(5)).await;
    }
}

#[embassy_executor::task]
async fn net_task() {
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(5)).await;
    }
}

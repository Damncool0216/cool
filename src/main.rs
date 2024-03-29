#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

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
use static_cell::StaticCell;

use atat::{asynch::Client, Config, Ingress, UrcChannel};
use atat::{AtatIngress, ResponseSlot};

use ec800m_at::gnss::types::{DeleteType, GnssConfig, NmeaConfig, NmeaType, Outport};
use ec800m_at::general::types::OnOff;
use ec800m_at::client::asynch::Ec800mClient;
use ec800m_at::digester::Ec800mDigester;
use ec800m_at::urc::URCMessages;

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
    let _delay = embassy_time::Delay;

    /* 串口初始化 */
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
    serial1.listen_at_cmd();
    serial1.listen_rx_fifo_full();
    let (writer, reader) = serial1.split();

    let config = atat::Config::default()
        .flush_timeout(Duration::from_millis(2000))
        .cmd_cooldown(Duration::from_millis(200))
        .tx_timeout(Duration::from_millis(2000));

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
    let client = Client::new(
        writer,
        &RES_SLOT,
        BUF.init([0; 1024]),
        Config::default(),
    );
    let ec800m_at_client: AtClient = Ec800mClient::new(client).await.unwrap();
    spawner.spawn(ingress_task(ingress, reader)).ok();
    spawner.spawn(at_client_task(ec800m_at_client)).ok();
    spawner.spawn(gnss_task()).ok();
    spawner.spawn(i2c_master_task()).ok();
}

#[embassy_executor::task]
async fn ingress_task(mut ingress: AtIngress<'static>, mut reader: AtReader<'static>) {
    ingress.read_from(&mut reader).await
}

#[embassy_executor::task]
async fn at_client_task(mut client: AtClient<'static>) {
    let mut state = 0;
    loop {
        state = state + 1;
        match state {
            1 => {
                if let Ok(_) = client.verify_com_is_working().await {
                    info!("com ok");
                } else {
                    info!("com fail");
                }
            }
            2 => {
                client.gpscfg_set_outport(Outport::None).await.ok();
                client.gpscfg_set_outport(Outport::UartDebug).await.ok();
                client.gpscfg_set_outport(Outport::UsbNmea).await.ok();
                
                client.gpscfg_set_nmea_src(OnOff::On).await.ok();
                client.gpscfg_set_nmea_src(OnOff::Off).await.ok();

                client.gpscfg_set_nmea_type(NmeaConfig::AllDisable).await.ok();
                client.gpscfg_set_nmea_type(NmeaConfig::AllEnable).await.ok();
                client.gpscfg_set_nmea_type(NmeaConfig::Defalut).await.ok();
                client.gpscfg_set_nmea_type(NmeaConfig::Config { gga: true, rmc: true, gsv: true, gsa: true, vtg: false, gll: false }).await.ok();

                client.gpscfg_set_gnss_config(GnssConfig::BeiDou).await.ok();
                client.gpscfg_set_gnss_config(GnssConfig::Gps).await.ok();
                client.gpscfg_set_gnss_config(GnssConfig::GpsBeiDou).await.ok();
                client.gpscfg_set_gnss_config(GnssConfig::GpsBeiDouGalileo).await.ok();
                client.gpscfg_set_gnss_config(GnssConfig::GpsGalileo).await.ok();
                client.gpscfg_set_gnss_config(GnssConfig::GpsGlonass).await.ok();
                client.gpscfg_set_gnss_config(GnssConfig::GpsGlonassGalileo).await.ok();

                client.gpscfg_set_auto_gps(OnOff::On).await.ok();
                client.gpscfg_set_auto_gps(OnOff::Off).await.ok();

                client.gpscfg_set_ap_flash(OnOff::On).await.ok();
                client.gpscfg_set_ap_flash(OnOff::Off).await.ok();

                client.gps_set_del(DeleteType::AllDel).await.ok();
                client.gps_set_del(DeleteType::NotDel).await.ok();
                client.gps_set_del(DeleteType::PartDel).await.ok();

                client.gps_set_sw(OnOff::On).await.ok();
                client.gps_set_sw(OnOff::Off).await.ok();

                client.gps_set_end().await.ok();

                client.gps_get_location().await.ok();

                client.gps_get_nmea(NmeaType::GGA).await.ok();
                client.gps_get_nmea(NmeaType::GLL).await.ok();
                client.gps_get_nmea(NmeaType::GSA).await.ok();
                client.gps_get_nmea(NmeaType::GSV).await.ok();
                client.gps_get_nmea(NmeaType::RMC).await.ok();
                client.gps_get_nmea(NmeaType::VTG).await.ok();

                client.gps_set_agps(OnOff::On).await.ok();
                client.gps_set_agps(OnOff::Off).await.ok();
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

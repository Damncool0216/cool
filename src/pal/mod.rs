use core::cell::RefCell;

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use hal::{
    clock::ClockControl,
    delay, embassy,
    gpio::IO,
    i2c::{self, I2C},
    peripherals::{Peripherals, I2C0},
    prelude::*,
    timer::TimerGroup,
    uart, Blocking,
};
use static_cell::StaticCell;

use crate::fml::storage::FmlGnssRawData;

pub mod flash;
pub mod gnss;
pub mod gsensor;
pub mod modem;
pub mod tsensor;

mod mlog;

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub(crate) enum Msg {
    None,
    //Gnss
    GnssMsgBegin,
    GnssOpenReq,
    GnssOpenRpy(bool),
    GnssCloseReq,
    GnssCloseRpy(bool),
    GnssGetLocationReq,
    GnssGetLoactionRpy(Option<FmlGnssRawData>),
    GnssMsgEnd,

    //net
    NetMsgBegin,
    NetSimStatReq,
    NetSimStatRpy(bool),
    NetAttachStatReq(Option<u8>),
    NetAttachStatRpy {
        stat: u8,
        lac: Option<u16>,
        ci: Option<u32>,
        act: Option<u8>,
    },
    NetMsgEnd,

    //Mqtt
    MqttMsgBegin,
    MqttOpenReq,
    MqttOpenRpy(bool),
    MqttConnReq,
    MqttConnRpy(bool),
    MqttPubReq,
    MqttPubRpy(bool),
    MqttCloseReq,
    MqttCloseRpy(bool),
    MqttEvent,
    MqttMsgEnd,
    //File

    //Modem
    ModemReady,
    ModemInitReq,

    //Tsensor
    TsensorMsgBegin,
    TsensorGetReq,
    TsensorGetRpy {
        temp: f32,
        humi: f32,
    },
    TsensorMsgEnd,

    //Gsensor,
    GsensorMsgBegin,
    GsensorVibEvent,
    GsensorMsgEnd,
}
pub(crate) type MsgQueue<const N: usize> = Channel<CriticalSectionRawMutex, Msg, N>;

static I2C_INIT: StaticCell<critical_section::Mutex<RefCell<I2C<'static, I2C0, Blocking>>>> =
    StaticCell::new();

static DELAY_INIT: StaticCell<delay::Delay> = StaticCell::new();

#[inline]
pub(crate) async fn msg_req(msg: Msg) {
    match msg {
        msg if (msg > Msg::GnssMsgBegin && msg < Msg::GnssMsgEnd)
            || (msg > Msg::MqttMsgBegin && msg < Msg::MqttMsgEnd)
            || (msg > Msg::NetMsgBegin && msg < Msg::NetMsgEnd) =>
        {
            modem::msg_req(msg).await
        }
        msg if msg > Msg::TsensorMsgBegin && msg < Msg::TsensorMsgEnd => {
            tsensor::msg_req(msg).await
        }
        _ => {}
    }
}

#[inline]
pub(self) async fn msg_rpy(msg: Msg) {
    crate::fml::msg_rpy(msg).await
}

/// init hardware
pub(super) fn init(spawner: &Spawner) {
    println::logger::init_logger_from_env();
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let timer_group0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timer_group0);

    //init uart at modem
    let config = uart::config::Config {
        baudrate: 115200,
        data_bits: uart::config::DataBits::DataBits8,
        parity: uart::config::Parity::ParityNone,
        stop_bits: uart::config::StopBits::STOP1,
        clock_source: uart::ClockSource::Apb,
    };

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let pins = uart::TxRxPins::new_tx_rx(
        io.pins.gpio0.into_push_pull_output(),
        io.pins.gpio1.into_floating_input(),
    );
    let mut serial1 =
        uart::Uart::new_async_with_config(peripherals.UART1, config, Some(pins), &clocks);
    serial1.set_rx_fifo_full_threshold(500).unwrap();
    serial1.set_at_cmd(uart::config::AtCmdConfig::new(
        Some(0),
        Some(0),
        None,
        b'\n',
        Some(1),
    )); //work!!! not shit now
    serial1.listen_at_cmd();
    serial1.listen_rx_fifo_full();

    let (writer, reader) = serial1.split();

    spawner.spawn(modem::pal_at_ingress_task(reader)).unwrap();
    spawner.spawn(modem::pal_at_urc_task()).unwrap();
    spawner.spawn(modem::pal_at_client_task(writer)).unwrap();

    //init i2c tsensor
    let i2c = i2c::I2C::new(
        peripherals.I2C0,
        io.pins.gpio4,
        io.pins.gpio5,
        1u32.MHz(),
        &clocks,
        None,
    );

    let i2c = I2C_INIT.init(critical_section::Mutex::new(RefCell::new(i2c)));
    let tsensor_i2c = embedded_hal_bus::i2c::CriticalSectionDevice::new(i2c);
    spawner
        .spawn(tsensor::pal_tsensor_task(
            delay::Delay::new(&clocks),
            tsensor_i2c,
        ))
        .unwrap();

    let gsensor_i2c = embedded_hal_bus::i2c::CriticalSectionDevice::new(i2c);
    let gensor_int2 = io.pins.gpio3.into_floating_input().degrade();

    spawner
        .spawn(gsensor::pal_gsensor_task(
            gsensor_i2c,
            Some(gensor_int2.into()),
        ))
        .unwrap();

    hal::interrupt::enable(
        hal::peripherals::Interrupt::GPIO,
        hal::interrupt::Priority::Priority1,
    )
    .unwrap();
}

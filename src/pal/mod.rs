use core::cell::RefCell;

use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{self, Channel},
};
use function_name::named;
use hal::{
    clock::ClockControl,
    delay, embassy,
    gpio::IO,
    i2c::{self, I2C},
    peripherals::{self, Peripherals, I2C0},
    prelude::*,
    timer::TimerGroup,
    uart, Blocking,
};
use static_cell::StaticCell;

use crate::mdebug;

pub mod gnss;
pub mod gsensor;
pub mod modem;
pub mod tsensor;

mod mlog;

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub(crate) enum PalMsg {
    None,
    //Gnss
    GnssMsgBegin,
    GnssOpenReq,
    GnssOpenRpy(bool),
    GnssCloseReq,
    GnssCloseRpy(bool),
    GnssGetLocationReq,
    GnssMsgEnd,
    //Mqtt

    //File

    //Tsensor
    TempHumiGetReq,
    TempHumiGetRpy { temp: f32, humi: f32 },
}

pub(self) type PalQueue<const N: usize> = Channel<CriticalSectionRawMutex, PalMsg, N>;

static PAL_TASK_QUEUE: PalQueue<30> = channel::Channel::new();

static I2C_INIT: StaticCell<critical_section::Mutex<RefCell<I2C<'static, I2C0, Blocking>>>> =
    StaticCell::new();

static DELAY_INIT: StaticCell<delay::Delay> = StaticCell::new();

#[inline]
pub(crate) async fn msg_req(msg: PalMsg) {
    match msg {
        msg if msg > PalMsg::GnssMsgBegin && msg < PalMsg::GnssMsgEnd => gnss::msg_req(msg).await,
        _ => {}
    }
}

#[inline]
pub(self) async fn msg_rpy(msg: PalMsg) {
    PAL_TASK_QUEUE.send(msg).await
}

#[embassy_executor::task()]
#[allow(unused_macros)]
#[named]
/// Handle the rpy, exec fml callback
async fn pal_msg_to_fml_task() {
    loop {
        let msg = PAL_TASK_QUEUE.receive().await;
        mdebug!("{:?}", msg);
        match msg {
            x if x > PalMsg::GnssMsgBegin && x <= PalMsg::GnssMsgEnd => {}
            PalMsg::TempHumiGetRpy { temp, humi } => {
                mdebug!("temp:{} °C humi:{} %RH", temp, humi);
            }
            _ => {}
        }
    }
}

/// init hardware
pub(crate) fn init(spawner: &Spawner) {
    println::logger::init_logger_from_env();
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let timer_group0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timer_group0);
    spawner.spawn(pal_msg_to_fml_task()).unwrap();

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
        Some(0), Some(0), None, b'\n', Some(1),
    )); //work!!! not shit now
    serial1.listen_at_cmd();
    serial1.listen_rx_fifo_full();

    let (writer, reader) = serial1.split();
    spawner.spawn(modem::pal_at_ingress_task(reader)).unwrap();
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

    // let gsensor_i2c = embedded_hal_bus::i2c::CriticalSectionDevice::new(i2c);

    // let mut spi = spi::master::Spi::new(peripherals.SPI2, 2u32.MHz(), SpiMode::Mode0, &clocks);
    // spi = spi::master::Spi::<_, FullDuplexMode>::with_miso(spi, io.pins.gpio10);
    // spi = spi::master::Spi::<_, FullDuplexMode>::with_mosi(spi, io.pins.gpio3);
    // spi = spi::master::Spi::<_, FullDuplexMode>::with_sck(spi, io.pins.gpio2);
    // let cs = io.pins.gpio7.into_open_drain_output();

    // spawner
    //     .spawn(gsensor::pal_gsensor_task(gsensor_i2c))
    //     .expect("what");
}

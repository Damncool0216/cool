use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{self, Channel},
};
use function_name::named;
use hal::{
    clock::ClockControl, embassy, gpio::IO, i2c, peripherals::Peripherals, prelude::*,
    timer::TimerGroup, uart,
};
use log::info;
pub mod gnss;
pub mod gsensor;
pub mod modem;
pub mod tsensor;

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum PalMsg {
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
}

pub type PalQueue<const N: usize> = Channel<CriticalSectionRawMutex, PalMsg, N>;

static PAL_TASK_QUEUE: PalQueue<30> = channel::Channel::new();

#[inline]
pub(crate) async fn msg_req(msg: PalMsg) {
    match msg {
        msg if msg > PalMsg::GnssMsgBegin && msg < PalMsg::GnssMsgEnd => gnss::msg_req(msg).await,
        _ => {}
    }
}

#[inline]
pub(crate) async fn msg_rpy(msg: PalMsg) {
    PAL_TASK_QUEUE.send(msg).await
}

#[embassy_executor::task()]
#[named]
/// Handle the rpy, exec fml callback
async fn pal_msg_to_fml_task() {
    loop {
        let msg = PAL_TASK_QUEUE.receive().await;
        info!("[{}] {:?}", function_name!(), msg);
        match msg {
            x if x > PalMsg::GnssMsgBegin && x <= PalMsg::GnssMsgEnd => {}
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
    serial1.set_rx_fifo_full_threshold(6).unwrap();
    serial1.set_at_cmd(uart::config::AtCmdConfig::new(
        None, None, None, b'\n', None,
    )); //not work! shit
    serial1.listen_at_cmd();
    serial1.listen_rx_fifo_full();
    let (writer, reader) = serial1.split();
    spawner.spawn(modem::ingress_task(reader)).unwrap();
    spawner.spawn(modem::client_task(writer)).unwrap();

    //init i2c tsensor
    let i2c = i2c::I2C::new(
        peripherals.I2C0,
        io.pins.gpio4,
        io.pins.gpio5,
        1u32.MHz(),
        &clocks,
        None,
    );

    //init spi gsensor
}

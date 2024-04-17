#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::cell::RefCell;

use drv::{drv_at_client_task, drv_at_ingress_task, drv_gnss::{drv_gnss_get_task_queue, drv_gnss_task, DrvGnssMsg}, drv_tsensor::drv_tsensor_task};

use esp_backtrace as _;
use embassy_executor::Spawner;
use hal::{
    clock::ClockControl, embassy, i2c, peripherals::Peripherals, prelude::*, timer::TimerGroup, uart, IO
};

mod drv;

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
    let pins = uart::TxRxPins::new_tx_rx(
        io.pins.gpio0.into_push_pull_output(),
        io.pins.gpio1.into_floating_input(),
    );
    let mut serial1 = uart::Uart::new_with_config(peripherals.UART1, config, Some(pins), &clocks);
    serial1.set_rx_fifo_full_threshold(6).unwrap();
    serial1.set_at_cmd(uart::config::AtCmdConfig::new(None, None, None, b'\r', None));
    serial1.listen_at_cmd();
    serial1.listen_rx_fifo_full();
    let (writer, reader) = serial1.split();

    let i2c = i2c::I2C::new(
        peripherals.I2C0,
        io.pins.gpio4,
        io.pins.gpio5,
        1u32.MHz(),
        &clocks,
    );

    // driver task
    //spawner.spawn(drv_at_ingress_task(reader)).ok();
    //spawner.spawn(drv_at_client_task(writer)).ok();
    //spawner.spawn(drv_net_task()).ok(); //todo
    //spawner.spawn(drv_gnss_task()).ok();
    spawner.spawn(drv_tsensor_task(i2c)).ok(); //todo
    //spawner.spawn(drv_gsensor_task()).ok(); //todo

    // app task
    //spawner.spawn(app_master_task()).ok(); //todo
}

#[embassy_executor::task]
async fn app_master_task() {
    loop {
        drv_gnss_get_task_queue().send(DrvGnssMsg::None).await;
        //drv_at_client_get_task_queue().send(DrvAtClientMsg::None).await;
        embassy_time::Timer::after(embassy_time::Duration::from_secs(2)).await;
        drv_gnss_get_task_queue().send(DrvGnssMsg::Open).await;
        embassy_time::Timer::after(embassy_time::Duration::from_secs(2)).await;
    }
}

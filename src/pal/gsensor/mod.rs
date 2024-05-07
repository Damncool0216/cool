use adxl345_driver2::{IntSource, TapMode};
use function_name::named;
use hal::{
    gpio::{AnyPin, Floating, Input},
    i2c::I2C,
    peripherals::I2C0,
    Blocking,
};

use crate::{debug, pal};

mod adxl345;
type I2cClient<'a> = embedded_hal_bus::i2c::CriticalSectionDevice<'a, I2C<'a, I2C0, Blocking>>;

/// Output scale is 4mg/LSB.
const SCALE_MULTIPLIER: f64 = 0.004;
/// Average Earth gravity in m/s²
const EARTH_GRAVITY_MS2: f64 = 9.80665;

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
pub(super) async fn pal_gsensor_task(
    i2c: I2cClient<'static>,
    mut int1: Option<AnyPin<Input<Floating>>>,
) {
    use adxl345_driver2::{self, Adxl345Reader, Adxl345Writer, IntControlMode, IntMapMode, Tap};
    let gsensor = adxl345_driver2::i2c::Device::new(i2c);
    if let Err(_) = gsensor {
        return;
    }
    let mut gsensor = gsensor.unwrap();

    debug!("device id:{:?}", gsensor.device_id());
    gsensor.set_data_format(0x0B).ok();
    gsensor.set_power_control(0x08).ok();
    gsensor.set_interrupt_map(IntMapMode::SINGLE_TAP_INT2).ok();

    gsensor.set_interrupt_control(IntControlMode::empty()).ok();
    gsensor.activity_tap_status().unwrap();
    gsensor
        .set_tap_control(TapMode::X_ENABLE | TapMode::Y_ENABLE | TapMode::Z_ENABLE)
        .ok();
    gsensor.set_tap(Tap::new(0x15, 0x10, 0x10, 0)).ok();

    gsensor
        .set_interrupt_control(IntControlMode::SINGLE_TAP_ENABLE)
        .ok();
    loop {
        if let Some(int1) = &mut int1 {
            gsensor.interrupt_source().unwrap();
            int1.wait_for_rising_edge().await;

            let int_src = gsensor.interrupt_source().unwrap();
            if int_src.contains(IntSource::SINGLE_TAP) {
                pal::msg_rpy(pal::Msg::GsensorVibEvent).await;
            }
            embassy_time::Timer::after_secs(1).await;
        }
    }
}

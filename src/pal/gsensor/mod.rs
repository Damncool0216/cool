use adxl345_driver2::{self};
use function_name::named;
use hal::{i2c::I2C, peripherals::I2C0, Blocking};

use crate::mdebug;

mod adxl345;
type I2cClient<'a> = embedded_hal_bus::i2c::CriticalSectionDevice<'a, I2C<'a, I2C0, Blocking>>;
//type Tsensor = ShtCx<Sht2Gen, I2cClient<'static>>;

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
pub(super) async fn pal_gsensor_task(i2c: I2cClient<'static>) {
    if let Err(e) = adxl345_driver2::i2c::Device::with_address(i2c, 0x1d) {
        mdebug!("{:?}", e);
    }
    // loop {
    //     if let Ok(s) = gsensor.device_id() {
    //         mdebug!("{}" , s);
    //     }
    //     if let Ok(s) = gsensor.acceleration() {
    //         mdebug!("{:?}", s);
    //     }
    //     embassy_time::Timer::after_secs(2).await;
    // }
}

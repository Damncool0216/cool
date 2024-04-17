use hal::{i2c::I2C, peripherals::I2C0};
use log::{debug, info};
use shtcx::sensor_class::Sht2Gen;
use shtcx::{LowPower, PowerMode, ShtCx};

type I2cClient<'a> = I2C<'a, I2C0>;
type Tsensor<'a> = ShtCx<Sht2Gen, I2cClient<'a>>;

async fn tsensor_wakeup(sht: &mut Tsensor<'static>) {
    sht.start_wakeup().unwrap();
    match sht.start_wakeup() {
        Ok(_) => {
            debug!("tsensor wakeup");
        }
        Err(_) => {}
    }
    embassy_time::Timer::after_micros(400).await;
}

#[embassy_executor::task]
pub async fn drv_tsensor_task(i2c: I2cClient<'static>) {
    let mut sht = shtcx::shtc3(i2c);
    tsensor_wakeup(&mut sht).await;
    let device_id = sht.device_identifier().unwrap();
    let raw_id = sht.raw_id_register().unwrap();
    debug!("sht device_id:{device_id}, raw_id:{raw_id}");
    loop {
        tsensor_wakeup(&mut sht).await;

        sht.start_measurement(PowerMode::LowPower).unwrap();
        embassy_time::Timer::after_micros(
            shtcx::max_measurement_duration(&sht, PowerMode::LowPower).into(),
        )
        .await;

        if let Ok(t) = sht.get_measurement_result() {
            info!(
                "temp:{} °C humi:{} %RH",
                t.temperature.as_degrees_celsius(),
                t.humidity.as_percent()
            );
        }
        match sht.sleep() {
            Ok(_) => {
                debug!("tsensor sleep");
            }
            Err(e) => {
                debug!("Error:{:?}", e);
            }
        }
        embassy_time::Timer::after_secs(60).await;
    }
}

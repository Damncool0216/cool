mod shtcx;

use embassy_sync::channel;
use function_name::named;
use hal::{delay, Blocking};
use hal::{i2c::I2C, peripherals::I2C0};

use shtcx::sensor_class::Sht2Gen;
use shtcx::{LowPower, Measurement, PowerMode, ShtCx};

use crate::{mdebug, minfo, pal};

use super::{Msg, MsgQueue};

type I2cClient<'a> = embedded_hal_bus::i2c::CriticalSectionDevice<'a, I2C<'a, I2C0, Blocking>>;
type Tsensor = ShtCx<Sht2Gen, I2cClient<'static>>;
static PAL_TSENSOR_TASK_QUEUE: MsgQueue<10> = channel::Channel::new();

#[inline]
pub(super) async fn msg_req(msg: Msg) {
    PAL_TSENSOR_TASK_QUEUE.send(msg).await
}

#[named]
async fn pal_tsensor_wakeup(sht: &mut Tsensor) {
    if let Ok(_) = sht.start_wakeup() {
        mdebug!("wakeup");
    }
    embassy_time::Timer::after_micros(400).await;
}

#[named]
fn pal_tsensor_sleep(sht: &mut Tsensor) {
    if let Ok(_) = sht.sleep() {
        mdebug!("sleep");
    }
}

async fn pal_temp_humi_get(
    sht: &mut Tsensor,
) -> Result<Measurement, shtcx::Error<hal::i2c::Error>> {
    let mut retry = 0;
    while let Err(e) = sht.start_measurement(PowerMode::LowPower) {
        retry += 1;
        if retry > 3 {
            return Err(e);
        }
    }

    embassy_time::Timer::after_micros(
        shtcx::max_measurement_duration(&sht, PowerMode::LowPower).into(),
    )
    .await;

    sht.get_measurement_result()
}

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
pub(super) async fn pal_tsensor_task(mut delay: delay::Delay, i2c: I2cClient<'static>) {
    let mut sht = shtcx::shtc3(i2c);
    pal_tsensor_wakeup(&mut sht).await;
    mdebug!(
        "device_id:{:?}, raw_id: {:?}",
        sht.device_identifier(),
        sht.raw_id_register()
    );
    loop {
        let msg = PAL_TSENSOR_TASK_QUEUE.receive().await;
        minfo!("{:?}", msg);
        pal_tsensor_wakeup(&mut sht).await;
        match msg {
            Msg::TsensorGetReq => {
                // if let Ok(t) = pal_temp_humi_get(&mut sht).await {
                //     pal::msg_rpy(PalMsg::TempHumiGetRpy {
                //         temp: t.temperature.as_degrees_celsius(),
                //         humi: t.humidity.as_percent(),
                //     })
                //     .await
                // }

                // blocking way, avoid confict with other i2c task
                if let Ok(t) = sht.measure(PowerMode::LowPower, &mut delay) {
                    pal::msg_rpy(Msg::TsensorGetRpy {
                        temp: t.temperature.as_degrees_celsius(),
                        humi: t.humidity.as_percent(),
                    })
                    .await
                }
            }
            _ => {}
        }
        pal_tsensor_sleep(&mut sht);
    }
}

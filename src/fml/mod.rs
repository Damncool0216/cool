use embassy_executor::Spawner;

use crate::pal;

pub mod alarm;
pub mod gnss;
pub mod mode;
pub mod net;
pub mod storage;
pub mod temp;

#[embassy_executor::task]
async fn temp_detect_task() {
    loop {
        pal::tsensor::get_temi_humi_req().await;
        embassy_time::Timer::after_secs(30).await
    }
}

pub fn init(spawner: &Spawner) {
    spawner.spawn(temp_detect_task()).unwrap();
}

#[embassy_executor::task]
async fn drv_gsensor_task() {
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(5)).await;
    }
}
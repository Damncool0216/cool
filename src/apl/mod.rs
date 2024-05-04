use embassy_executor::Spawner;

pub mod master;

#[embassy_executor::task]
pub(super) async fn apl_master_task() {
    loop {
        embassy_time::Timer::after_secs(100000).await;
    }
}
pub fn init(spawner: &Spawner) {
    spawner.spawn(apl_master_task()).unwrap();
}

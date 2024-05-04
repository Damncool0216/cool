pub mod alarm;
pub mod gnss;
pub mod mode;
pub mod net;
pub mod storage;
pub mod temp;

pub fn init(spawner: &embassy_executor::Spawner) {
    spawner.spawn(temp::fml_temp_detect_task()).unwrap();
}

use crate::{
    info,
    pal::{Msg, MsgQueue},
};
use core::sync::atomic::{AtomicU8, Ordering};
use embassy_sync::channel;
use function_name::named;

use super::FmlAccStatus;

static FML_ACC_MSG_QUEUE: MsgQueue<10> = channel::Channel::new();
static ACC_STATUS: AtomicU8 = AtomicU8::new(0);

#[named]
fn fml_acc_status_set(new_status: FmlAccStatus) {
    if fml_acc_status_get() == new_status {
        return;
    }
    ACC_STATUS.store(new_status as u8, Ordering::Relaxed);
    if new_status != FmlAccStatus::Off {
        info!("static to sport");
    } else {
        info!("sport to static");
    }
}

pub fn fml_acc_status_get() -> FmlAccStatus {
    ACC_STATUS.load(Ordering::Relaxed).into()
}

pub fn fml_acc_is_on() -> bool {
    fml_acc_status_get() == FmlAccStatus::On
}

#[inline]
pub(super) async fn msg_rpy(msg: Msg) {
    FML_ACC_MSG_QUEUE.send(msg).await
}

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
pub(super) async fn fml_acc_msg_rpy_task() {
    let mut vib_cnt = 0;
    let mut last_vib_time = embassy_time::Instant::now();
    loop {
        let mut msg = None;
        let is_acc_on = fml_acc_status_get() == FmlAccStatus::On;
        if is_acc_on {
            if let Ok(s) = FML_ACC_MSG_QUEUE.try_receive() {
                msg = Some(s);
            } else {
                if last_vib_time.elapsed() > embassy_time::Duration::from_secs(3 * 60) {
                    fml_acc_status_set(FmlAccStatus::Off);
                }
                embassy_time::Timer::after_secs(5).await;
                continue;
            }
        }
        if let None = msg {
            msg = Some(FML_ACC_MSG_QUEUE.receive().await);
        }
        info!("{:?}", msg);
        match msg.unwrap() {
            Msg::GsensorVibEvent => {
                if last_vib_time.elapsed() > embassy_time::Duration::from_secs(10) {
                    vib_cnt = 0;
                }
                if vib_cnt < 3 {
                    vib_cnt = vib_cnt + 1;
                } else {
                    fml_acc_status_set(FmlAccStatus::On);
                }
                last_vib_time = embassy_time::Instant::now();
                info!("vib_time:{}, vib cnt:{}", last_vib_time, vib_cnt);
            }
            _ => {}
        }
    }
}

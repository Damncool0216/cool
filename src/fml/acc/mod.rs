use core::sync::atomic::{AtomicU8, Ordering};

use embassy_sync::channel;
use function_name::named;

use crate::{
    minfo,
    pal::{Msg, MsgQueue},
};

static FML_ACC_MSG_QUEUE: MsgQueue<10> = channel::Channel::new();
#[inline]
pub(super) async fn msg_rpy(msg: Msg) {
    FML_ACC_MSG_QUEUE.send(msg).await
}
static ACC_STATUS: AtomicU8 = AtomicU8::new(0);

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
pub(super) async fn fml_acc_msg_rpy_task() {
    let mut vib_cnt = 0;
    let mut last_vib_time = embassy_time::Instant::now();
    ACC_STATUS.store(0, Ordering::Relaxed);
    loop {
        let mut msg = None;
        if ACC_STATUS.load(Ordering::Relaxed) != 0 {
            if let Ok(s) = FML_ACC_MSG_QUEUE.try_receive() {
                msg = Some(s);
            } else {
                if last_vib_time.elapsed() > embassy_time::Duration::from_secs(3 * 60) {
                    ACC_STATUS.store(0, Ordering::Relaxed);
                    minfo!("sport to static");
                }
                embassy_time::Timer::after_secs(5).await;
                continue;
            }
        }
        if let None = msg {
            msg = Some(FML_ACC_MSG_QUEUE.receive().await);
        }
        minfo!("{:?}", msg);
        match msg.unwrap() {
            Msg::GsensorVibEvent => {
                if last_vib_time.elapsed() > embassy_time::Duration::from_secs(10) {
                    vib_cnt = 0;
                }
                if vib_cnt < 3 {
                    vib_cnt = vib_cnt + 1;
                } else {
                    if ACC_STATUS.load(Ordering::Relaxed) == 0 {
                        ACC_STATUS.store(1, Ordering::Relaxed);
                        minfo!("static to sport");
                    }
                }
                last_vib_time = embassy_time::Instant::now();
                minfo!("vib_time:{}, vib cnt:{}", last_vib_time, vib_cnt);
            }
            _ => {}
        }
    }
}

pub(crate) fn fml_acc_is_on() -> bool {
    ACC_STATUS.load(Ordering::Relaxed) == 1
}

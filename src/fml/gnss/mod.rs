use crate::pal::Msg;

#[inline]
pub(super) async fn msg_rpy(_msg: Msg) {
    embassy_time::Timer::after_secs(1).await
}

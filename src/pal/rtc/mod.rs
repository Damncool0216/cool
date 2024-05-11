use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use function_name::named;
use hal::rtc_cntl::Rtc;

use crate::info;

static RTC_HANDLE: Mutex<CriticalSectionRawMutex,Option<Rtc>> = Mutex::new(None);

pub async fn pal_rtc_get_time_ms() -> u64 {
    let mut rtc_time_ms = 0;
    if let Some(r) = &mut *RTC_HANDLE.lock().await {
        rtc_time_ms = r.get_time_ms();
    }
    return rtc_time_ms;
}

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
pub async fn pal_rtc_init_task(rtc: Rtc<'static>) {
    let s = &mut *RTC_HANDLE.lock().await;
    *s = Some(rtc);
    info!("rtc init");
}
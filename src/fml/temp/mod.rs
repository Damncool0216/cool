use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use function_name::named;

use crate::{mdebug, pal};

async fn get_temp_humi_req() {
    pal::msg_req(pal::PalMsg::TempHumiGetReq).await
}

#[derive(Debug, Clone, Copy)]
struct FmLTempNvm {
    /// temp humi detect interval min
    detect_inv: u32,
}

impl FmLTempNvm {
    pub fn default() -> Self {
        Self { detect_inv: 1 }
    }
}

// store fml temp task pararm config
static FML_TEMP_NVM: Mutex<CriticalSectionRawMutex, Option<FmLTempNvm>> = Mutex::new(None);

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
pub(super) async fn fml_temp_detect_task() {
    {
        *(FML_TEMP_NVM.lock().await) = Some(FmLTempNvm::default())
    }
    loop {
        //get_temp_humi_req().await;
        let mut after = embassy_time::Timer::after_secs(1 * 60);
        if let Some(fml_temp_nvm) = &*FML_TEMP_NVM.lock().await {
            // not await in lock
            mdebug!("{:?}", fml_temp_nvm);
            after = embassy_time::Timer::after_secs(fml_temp_nvm.detect_inv as u64 * 60);
        }
        after.await
    }
}

// #[embassy_executor::task]
// pub(super) async fn fml_change_detect_task() {
//     let mut i = 0;
//     loop {
//         i = i + 1;
//         if let Some(fml_temp_nvm) = &mut *FML_TEMP_NVM.lock().await {
//             fml_temp_nvm.detect_inv = i;
//         }
//         embassy_time::Timer::after_secs(1).await
//     }
// }

use embassy_sync::channel;
use embassy_time::Duration;
use function_name::named;

use crate::{
    debug,
    fml::storage::*,
    info,
    pal::{self, Msg, MsgQueue},
};

static FML_TEMP_MSG_QUEUE: MsgQueue<10> = channel::Channel::new();

#[inline]
pub async fn fml_temp_humi_get_req() {
    pal::msg_req(pal::Msg::TsensorGetReq).await
}

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
pub(super) async fn fml_temp_detect_task() {
    let mut detect_inv = 1;
    let mut ticker = embassy_time::Ticker::every(Duration::from_secs(detect_inv as u64 * 60));
    loop {
        if let Ok(fml_temp_nvm) = FML_TEMP_NVM.try_lock() {
            // not await in lock
            if let Some(fml_temp_nvm) = &*fml_temp_nvm {
                debug!("{:?}", fml_temp_nvm);
                if fml_temp_nvm.detect_inv != detect_inv {
                    ticker = embassy_time::Ticker::every(Duration::from_secs(
                        fml_temp_nvm.detect_inv as u64 * 60,
                    ));
                    detect_inv = fml_temp_nvm.detect_inv;
                }
            }
        }
        if detect_inv > 0 {
            fml_temp_humi_get_req().await;
        }
        ticker.next().await;
    }
}

#[inline]
pub(super) async fn msg_rpy(msg: Msg) {
    FML_TEMP_MSG_QUEUE.send(msg).await
}

#[embassy_executor::task]
#[allow(unused_macros)]
#[named]
pub(super) async fn fml_temp_msg_rpy_task(mut producer: FmlTempHumiProducer<'static>) {
    loop {
        let msg = FML_TEMP_MSG_QUEUE.receive().await;
        info!("{:?}", msg);
        match msg {
            Msg::TsensorGetRpy { temp, humi } => {
                info!("temp:{} °C humi:{} %RH", temp, humi);
                let data = FmlTempHumiData::new(1, temp, humi);
                if producer.ready() {
                    producer.enqueue(data).ok();
                    info!(
                        "enqueue temp humi: {}/{}",
                        producer.len(),
                        producer.capacity()
                    );
                    super::net::fml_net_mqtt_pub_req().await;
                }
            }
            _ => {}
        }
    }
}

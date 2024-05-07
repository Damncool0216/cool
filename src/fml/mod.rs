use self::storage::*;
use crate::pal::Msg;
use static_cell::StaticCell;

pub mod acc;
pub mod alarm;
pub mod gnss;
pub mod net;
pub mod storage;
pub mod temp;

pub(super) fn init(spawner: &embassy_executor::Spawner) {
    static TEMP_QUEUE_INIT: StaticCell<FmlTempHumiQueue> = StaticCell::new();
    let (temp_p, temp_c) = TEMP_QUEUE_INIT.init(FmlTempHumiQueue::new()).split();

    spawner.spawn(storage::fml_storage_task()).unwrap();

    spawner.spawn(temp::fml_temp_msg_rpy_task(temp_p)).unwrap();
    spawner.spawn(temp::fml_temp_detect_task()).unwrap();

    spawner.spawn(net::fml_net_status_task()).unwrap();
    spawner.spawn(net::fml_net_recv_task()).unwrap();
    spawner.spawn(net::fml_net_send_task(temp_c)).unwrap();

    spawner.spawn(acc::fml_acc_msg_rpy_task()).unwrap();
}

#[inline]
pub(crate) async fn msg_rpy(msg: Msg) {
    match msg {
        msg if msg > Msg::TsensorMsgBegin && msg < Msg::TsensorMsgEnd => temp::msg_rpy(msg).await,
        msg if msg > Msg::GsensorMsgBegin && msg < Msg::GsensorMsgEnd => acc::msg_rpy(msg).await,
        msg if msg > Msg::NetMsgBegin && msg < Msg::NetMsgEnd => net::msg_rpy(msg).await,
        msg if msg > Msg::MqttMsgBegin && msg < Msg::MqttMsgEnd => net::msg_rpy(msg).await,
        msg if msg > Msg::GnssMsgBegin && msg < Msg::GnssMsgEnd => gnss::msg_rpy(msg).await,
        _ => {}
    }
}

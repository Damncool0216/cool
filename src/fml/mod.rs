use serde_json_core::heapless::spsc::Queue;
use static_cell::StaticCell;

use self::storage::*;
use crate::pal::Msg;

pub mod acc;
pub mod alarm;
pub mod gnss;
pub mod net;
pub mod storage;
pub mod temp;

pub type FmlDataQueue<T> = serde_json_core::heapless::spsc::Queue<T, 128>;
pub type FmlDataProducer<'a, T> = serde_json_core::heapless::spsc::Producer<'a, T, 128>;
pub type FmlDataComsumer<'a, T> = serde_json_core::heapless::spsc::Consumer<'a, T, 128>;

pub(super) fn init(spawner: &embassy_executor::Spawner) {
    static T_INIT: StaticCell<FmlDataQueue<FmlTempHumiData>> = StaticCell::new();
    static G_INIT: StaticCell<FmlDataQueue<FmlGnssData>> = StaticCell::new();
    let (temp_p, temp_c) = T_INIT.init(Queue::new()).split();
    let (gnss_p, gnss_c) = G_INIT.init(Queue::new()).split();
    spawner.spawn(storage::fml_storage_task()).unwrap();

    spawner.spawn(temp::fml_temp_msg_rpy_task(temp_p)).unwrap();
    spawner.spawn(temp::fml_temp_detect_task()).unwrap();

    spawner.spawn(net::fml_net_status_task()).unwrap();
    spawner.spawn(net::fml_net_recv_task()).unwrap();
    spawner.spawn(net::fml_net_send_task(temp_c, gnss_c)).unwrap();

    spawner.spawn(gnss::fml_gnss_control_task()).unwrap();
    spawner.spawn(gnss::fml_gnss_data_filter_task(gnss_p)).unwrap();

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
        msg if msg == Msg::ModemReady => {
            gnss::msg_rpy(Msg::ModemReady).await;
            net::msg_rpy(Msg::ModemReady).await;
        }
        _ => {}
    }
}

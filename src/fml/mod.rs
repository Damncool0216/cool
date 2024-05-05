use crate::pal::Msg;

pub mod acc;
pub mod alarm;
pub mod gnss;
pub mod net;
pub mod storage;
pub mod temp;

pub(super) fn init(spawner: &embassy_executor::Spawner) {
    spawner.spawn(temp::fml_temp_msg_rpy_task()).unwrap();
    spawner.spawn(temp::fml_temp_detect_task()).unwrap();

    spawner.spawn(net::fml_net_attach_task()).unwrap();
    spawner.spawn(net::fml_net_connect_task()).unwrap();
    spawner.spawn(net::fml_net_recv_task()).unwrap();
    spawner.spawn(net::fml_net_send_task()).unwrap();

    spawner.spawn(acc::fml_acc_msg_rpy_task()).unwrap();
}

#[inline]
pub(crate) async fn msg_rpy(msg: Msg) {
    match msg {
        msg if msg > Msg::TsensorMsgBegin && msg < Msg::TsensorMsgEnd => temp::msg_rpy(msg).await,
        msg if msg > Msg::GsensorMsgBegin && msg < Msg::GsensorMsgEnd => acc::msg_rpy(msg).await,
        _ => {}
    }
}

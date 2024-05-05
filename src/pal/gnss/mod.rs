use super::{modem, Msg};

#[inline]
pub(super) async fn msg_req(msg: Msg) {
    modem::msg_req(msg).await
}

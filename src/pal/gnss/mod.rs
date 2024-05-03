use super::{modem, PalMsg};

pub async fn open_req() {
    msg_req(PalMsg::GnssOpenReq).await
}

#[inline]
pub(crate) async fn msg_req(msg: PalMsg) {
    modem::msg_req(msg).await
}
pub mod cmds;
pub mod resps;
pub mod types;

#[cfg(feature = "async")]
pub mod asynch {
    use super::super::client::asynch::Ec800mClient as Client;
    use super::cmds::*;
    use super::types::*;
    use atat::asynch::AtatClient;
    use atat::Error;
    use embedded_io_async::Write;
    use log::debug;
    use serde_json_core::heapless::String;

    impl<'a, W: Write, const INGRESS_BUF_SIZE: usize> Client<'a, W, INGRESS_BUF_SIZE> {
        pub async fn mqtt_open(
            &mut self,
            client_idx: MqttClientIdx,
            host_name: &str,
            port: u16,
        ) -> Result<bool, Error> {
            let cmd = QMtOpen::new(client_idx, host_name, port);
            let resp = self.client.send(&cmd).await?;
            debug!("{:?}", resp);
            Ok(resp.is_ok())
        }
        pub async fn mqtt_close(&mut self, client_idx: MqttClientIdx) -> Result<bool, Error> {
            let cmd = QMtClose::new(client_idx);
            let resp = self.client.send(&cmd).await?;
            debug!("{:?}", resp);
            Ok(resp.is_ok())
        }
        pub async fn mqtt_conn(
            &mut self,
            client_idx: MqttClientIdx,
            clinet_id: &str,
            username: &str,
            password: &str,
        ) -> Result<bool, Error> {
            let cmd = QMtConn::new(client_idx, clinet_id, Some(username), Some(password));
            let resp = self.client.send(&cmd).await?;
            debug!("{:?}", resp);
            Ok(resp.is_ok())
        }
        pub async fn mqtt_version_config(
            &mut self,
            client_idx: MqttClientIdx,
            vsn: MqttVersion,
        ) -> Result<bool, Error> {
            let cmd = QMtCfgVersionSet::new(client_idx, vsn);
            let resp = self.client.send(&cmd).await?;
            debug!("{:?}", resp);
            Ok(resp.is_ok())
        }

        pub async fn mqtt_publish(
            &mut self,
            client_idx: MqttClientIdx,
            msgid: u16,
            qos: MqttQos,
            retain: u8,
            topic: String<50>,
            data: String<512>,
        ) -> Result<bool, Error> {
            let cmd = QMtPubEx::new(client_idx, msgid, qos, retain, &topic, data.len() as u16);
            debug!("{:?}", cmd);
            let resp = self.client.send(&cmd).await?;
            debug!("{:?}", resp);
            if resp.is_send_ready() {
                self.send_data(&data).await
            } else {
                Ok(false)
            }
        }

        async fn send_data(&mut self, data: &str) -> Result<bool, Error> {
            let cmd = SendData { data };
            debug!("{:?}", cmd);
            let resp = self.client.send(&cmd).await?;
            debug!("{:?}", resp);
            Ok(true)
        }
    }
}

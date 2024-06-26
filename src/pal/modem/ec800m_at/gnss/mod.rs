pub mod cmds;
pub mod resps;
pub mod types;

#[cfg(feature = "async")]
pub mod asynch {
    use crate::pal::modem::ec800m_at::gnss::resps::GpsLocResp;

    use super::super::client::asynch::Ec800mClient as Client;
    use super::super::general::types::OnOff;
    use super::super::gnss::cmds::QGpsSet;
    use super::cmds::{
        QAgpsSet, QGpsCfgApFlashSet, QGpsCfgAutoGpsSet, QGpsCfgGnssConfigSet,
        QGpsCfgGpsNmeaTypeSet, QGpsCfgNmeasrcSet, QGpsCfgOutPortSet, QGpsDelSet, QGpsEndSet,
        QGpsLoc, QGpsNmeaGet,
    };
    use super::types::{DeleteType, GnssConfig, NmeaConfig, NmeaType, NmeaVec, Outport};
    use atat::asynch::AtatClient;
    use atat::Error;
    use embedded_io_async::Write;
    use log::debug;

    impl<'a, W: Write, const INGRESS_BUF_SIZE: usize> Client<'a, W, INGRESS_BUF_SIZE> {
        pub async fn gpscfg_set_outport(&mut self, port: Outport) -> Result<bool, Error> {
            let cmd = QGpsCfgOutPortSet::new(port);
            let resp = self.client.send(&cmd).await?;
            Ok(resp.is_ok())
        }
        pub async fn gpscfg_set_nmea_src(&mut self, on_off: OnOff) -> Result<bool, Error> {
            let cmd = QGpsCfgNmeasrcSet::new(on_off);
            let resp = self.client.send(&cmd).await?;
            Ok(resp.is_ok())
        }
        pub async fn gpscfg_set_nmea_type(&mut self, nmea_type: NmeaConfig) -> Result<bool, Error> {
            let cmd = QGpsCfgGpsNmeaTypeSet::new(nmea_type);
            let resp = self.client.send(&cmd).await?;
            Ok(resp.is_ok())
        }
        pub async fn gpscfg_set_gnss_config(
            &mut self,
            gnss_config: GnssConfig,
        ) -> Result<bool, Error> {
            let cmd = QGpsCfgGnssConfigSet::new(gnss_config);
            let resp = self.client.send(&cmd).await?;
            Ok(resp.is_ok())
        }
        pub async fn gpscfg_set_auto_gps(&mut self, on_off: OnOff) -> Result<bool, Error> {
            let cmd = QGpsCfgAutoGpsSet::new(on_off);
            let resp = self.client.send(&cmd).await?;
            Ok(resp.is_ok())
        }
        pub async fn gpscfg_set_ap_flash(&mut self, on_off: OnOff) -> Result<bool, Error> {
            let cmd = QGpsCfgApFlashSet::new(on_off);
            let resp = self.client.send(&cmd).await?;
            Ok(resp.is_ok())
        }
        pub async fn gps_set_del(&mut self, delete_type: DeleteType) -> Result<bool, Error> {
            let cmd = QGpsDelSet::new(delete_type);
            let resp = self.client.send(&cmd).await?;
            Ok(resp.is_ok())
        }
        pub async fn gps_set_sw(&mut self, on_off: OnOff) -> Result<bool, Error> {
            let cmd = QGpsSet::new(on_off);
            let resp = self.client.send(&cmd).await?;
            Ok(resp.is_ok())
        }
        pub async fn gps_set_end(&mut self) -> Result<bool, Error> {
            let cmd = QGpsEndSet;
            let resp = self.client.send(&cmd).await?;
            Ok(resp.is_ok())
        }
        pub async fn gps_set_agps(&mut self, on_off: OnOff) -> Result<bool, Error> {
            let cmd = QAgpsSet::new(on_off);
            let resp = self.client.send(&cmd).await?;
            Ok(resp.is_ok())
        }
        pub async fn gps_get_location(&mut self) -> Result<GpsLocResp, Error> {
            let cmd = QGpsLoc::default();
            let resp = self.client.send(&cmd).await?;
            debug!("{:?}", resp);
            Ok(resp)
        }
        pub async fn gps_get_nmea(&mut self, nmea_type: NmeaType) -> Result<NmeaVec, Error> {
            let cmd = QGpsNmeaGet::new(nmea_type);
            let resp = self.client.send(&cmd).await?;
            Ok(resp.nmeas)
        }
    }
}

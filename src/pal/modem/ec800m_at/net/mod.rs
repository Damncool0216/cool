pub mod cmds;
pub mod resps;
pub mod types;

#[cfg(feature = "async")]
pub mod asynch {
    use super::super::client::asynch::Ec800mClient as Client;
    use super::cmds::*;
    use atat::asynch::AtatClient;
    use atat::Error;
    use embedded_io_async::Write;
    use log::debug;

    impl<'a, W: Write, const INGRESS_BUF_SIZE: usize> Client<'a, W, INGRESS_BUF_SIZE> {
        pub async fn sim_query(&mut self) -> Result<bool, Error> {
            let cmd = CpinQuery;
            let resp = self.client.send(&cmd).await?;
            debug!("{:?}", resp);
            Ok(resp.is_ready())
        }
        pub async fn creg_query(
            &mut self,
        ) -> Result<(u8, Option<u16>, Option<u32>, Option<u8>), Error> {
            let cmd = CregQuery;
            let resp = self.client.send(&cmd).await?;
            debug!("{:?}", resp);
            Ok((
                resp.stat,
                resp.lac
                    .map(|f| u16::from_str_radix(f.as_str(), 16).unwrap()),
                resp.ci
                    .map(|f| u32::from_str_radix(f.as_str(), 16).unwrap()),
                resp.act,
            ))
        }

        pub async fn creg_set(&mut self, n: u8) -> Result<bool, Error> {
            let cmd = CregSet { n };
            let resp = self.client.send(&cmd).await?;
            debug!("{:?}", resp);
            Ok(resp.is_ok())
        }
    }
}

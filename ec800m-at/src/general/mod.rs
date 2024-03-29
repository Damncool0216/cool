pub mod cmds;
pub mod resps;
pub mod types;

#[cfg(feature = "async")]
pub mod asynch {
    use crate::client::asynch::Ec800mClient as Client;
    use crate::general::cmds::{AteSet, VerifyAT};
    use atat::asynch::AtatClient;
    use atat::Error;
    use embedded_io_async::Write;
    use log::{debug, info};

    impl<'a, W: Write, const INGRESS_BUF_SIZE: usize> Client<'a, W, INGRESS_BUF_SIZE> {
        pub async fn verify_com_is_working(&mut self) -> Result<bool, Error> {
            let cmd = VerifyAT;
            let resp = self.client.send(&cmd).await?;
            #[cfg(feature = "debug")]
            info!("{:?}", resp);
            Ok(resp.is_ok())
        }
        
        pub async fn at_echo_set(&mut self, on_off: bool) -> Result<bool, Error> {
            let command = if on_off {AteSet::on()} else {AteSet::off()};
            #[cfg(feature = "debug")]
            info!("{:?}", command);
            let response = self.client.send(&command).await?;
            Ok(response.is_ok())
        }
    }
}

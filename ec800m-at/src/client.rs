#[cfg(feature = "async")]
pub mod asynch {
    pub use atat::asynch::Client;
    use atat::Error;
    pub use embedded_io_async::Write;
    #[cfg(feature = "debug")]
    use log::{error, info};

    pub struct Ec800mClient<'a, W: Write, const INGRESS_BUF_SIZE: usize> {
        pub(crate) client: Client<'a, W, INGRESS_BUF_SIZE>,
    }

    impl<'a, W: Write, const INGRESS_BUF_SIZE: usize> Ec800mClient<'a, W, INGRESS_BUF_SIZE> {
        pub async fn new(
            client: Client<'a, W, INGRESS_BUF_SIZE>,
        ) -> Result<Ec800mClient<'a, W, INGRESS_BUF_SIZE>, Error> {
            let mut s = Self { client };
            
            if s.at_echo_set(false).await.is_err() {
                #[cfg(feature = "debug")]
                error!("set echo to false falid");
            }
            
            Ok(s)
        }
    }
}

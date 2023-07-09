#[cfg(feature = "async")]
pub mod asynch {
    pub use atat::asynch::Client;
    use atat::Error;
    use defmt::error;
    pub use embedded_io::asynch::Write;

    pub struct SeeedLoraE5Client<'a, W: Write, const INGRESS_BUF_SIZE: usize> {
        pub(crate) client: Client<'a, W, INGRESS_BUF_SIZE>,
    }

    impl<'a, W: Write, const INGRESS_BUF_SIZE: usize> SeeedLoraE5Client<'a, W, INGRESS_BUF_SIZE> {
        pub async fn new(
            client: Client<'a, W, INGRESS_BUF_SIZE>,
        ) -> Result<SeeedLoraE5Client<'a, W, INGRESS_BUF_SIZE>, Error> {
            let mut s = Self { client };
            if s.reset().await.is_err() {
                error!("Error resetting Moko");
            }
            if s.at_echo_set(false).await.is_err() {
                error!("Error settign echo to false");
            }
            Ok(s)
        }
    }
}

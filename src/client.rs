#[cfg(feature = "async")]
pub mod asynch {
    pub use atat::asynch::Client;
    use atat::Error;
    use defmt::{error, info, warn};
    pub use embedded_io::asynch::Write;

    pub struct SeeedLoraE5Client<'a, W: Write, const INGRESS_BUF_SIZE: usize> {
        pub(crate) client: Client<'a, W, INGRESS_BUF_SIZE>,
    }

    impl<'a, W: Write, const INGRESS_BUF_SIZE: usize> SeeedLoraE5Client<'a, W, INGRESS_BUF_SIZE> {
        pub async fn new(
            client: Client<'a, W, INGRESS_BUF_SIZE>,
        ) -> Result<SeeedLoraE5Client<'a, W, INGRESS_BUF_SIZE>, Error> {
            let mut s = Self { client };
            if let Err(e) = s.verify_com_is_working().await {
                error!("Error verifying Seeed LoRa-E5 comms: {:?}", e);
            }
            if s.reset().await.is_err() {
                error!("Error resetting Seeed LoRa-E5");
            }
            while s.verify_com_is_working().await.is_err() {
                warn!("Waiting of LoRa-E5 to reset...");
            }
            let version = s.version().await;
            match version {
                Err(e) => error!("Error getting Seeed LoRa-E5 firmware version: {:?}", e),
                Ok((major, minor, patch)) => info!("Seeed LoRa-E5 firmware version: {}.{}.{}", major, minor, patch),
            }
            Ok(s)
        }
    }
}

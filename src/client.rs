#[cfg(feature = "async")]
pub mod asynch {
    use crate::general::responses::VerResponse;
    pub use atat::asynch::Client;
    use atat::{Error};
    #[cfg(feature = "debug")]
    use defmt::{error, info, warn};
    pub use embedded_io::asynch::Write;
    use heapless::String;

    #[derive(Clone, Debug, Copy)]
    pub enum JoinStatus {
        Joining,
        Success,
        Failure,
        NotJoined,
        Unknown,
    }

    pub struct OtaaJoinStatus {
        pub join_status: JoinStatus,
        pub net_id: Option<String<12>>,
        pub dev_addr: Option<String<22>>,
    }

    pub struct SeeedLoraE5Client<'a, W: Write, const INGRESS_BUF_SIZE: usize> {
        pub(crate) client: Client<'a, W, INGRESS_BUF_SIZE>,
        pub(crate) join_status: OtaaJoinStatus,
    }

    impl<'a, W: Write, const INGRESS_BUF_SIZE: usize> SeeedLoraE5Client<'a, W, INGRESS_BUF_SIZE> {
        pub async fn new(
            client: Client<'a, W, INGRESS_BUF_SIZE>,
        ) -> Result<SeeedLoraE5Client<'a, W, INGRESS_BUF_SIZE>, Error> {
            let mut s = Self {
                client,
                join_status: OtaaJoinStatus {
                    join_status: JoinStatus::NotJoined,
                    net_id: None,
                    dev_addr: None,
                },
            };

            if let Err(e) = s.verify_com_is_working().await {
                #[cfg(feature = "debug")]
                error!("Error verifying Seeed LoRa-E5 comms: {:?}", e);
            }
            if s.reset().await.is_err() {
                #[cfg(feature = "debug")]
                error!("Error resetting Seeed LoRa-E5");
            }
            let mut count_down = 10;
            while s.verify_com_is_working().await.is_err() && count_down > 0 {
                #[cfg(feature = "debug")]
                warn!("Waiting for LoRa-E5 to reset...");
                count_down -= 1;
            }
            if count_down == 0 {
                return Err(Error::Timeout);
            }
            let version = s.version().await;
            match version {
                Err(e) => {
                    #[cfg(feature = "debug")]
                    error!("Error getting Seeed LoRa-E5 firmware version: {:?}", e);
                }
                Ok(VerResponse {
                    major,
                    minor,
                    patch,
                }) => {
                    #[cfg(feature = "debug")]
                    info!(
                        "Seeed LoRa-E5 firmware version: {}.{}.{}",
                        major, minor, patch
                    );
                }
            }
            Ok(s)
        }
    }
}

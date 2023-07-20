#[cfg(feature = "async")]
pub mod asynch {
    use crate::general::responses::VerResponse;
    use crate::lora::urc::JoinUrc;
    use crate::urc::URCMessages;
    pub use atat::asynch::Client;
    use atat::{Error, UrcSubscription};
    #[cfg(feature = "debug")]
    use defmt::{error, info, warn};
    use embassy_sync::pubsub::WaitResult;
    pub use embedded_io::asynch::Write;
    use heapless::String;

    #[derive(Clone, Debug)]
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
        pub(crate) urc_subscription: UrcSubscription<'a, URCMessages>,
        pub(crate) join_status: OtaaJoinStatus,
    }

    impl<'a, W: Write, const INGRESS_BUF_SIZE: usize> SeeedLoraE5Client<'a, W, INGRESS_BUF_SIZE> {
        pub async fn new(
            client: Client<'a, W, INGRESS_BUF_SIZE>,
            urc_subscription: UrcSubscription<'a, URCMessages>,
        ) -> Result<SeeedLoraE5Client<'a, W, INGRESS_BUF_SIZE>, Error> {
            let mut s = Self {
                client,
                urc_subscription,
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
            while s.verify_com_is_working().await.is_err() {
                #[cfg(feature = "debug")]
                warn!("Waiting of LoRa-E5 to reset...");
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

        pub async fn handle_urc(&mut self) -> Result<Option<URCMessages>, Error> {
            let msg = self.urc_subscription.try_next_message();
            if msg.is_none() {
                return Ok(None);
            }
            let msg = msg.unwrap();
            match msg {
                WaitResult::Lagged(amount) => {
                    #[cfg(feature = "debug")]
                    warn!("Missed URC messages {}", amount);
                    Ok(None)
                }
                WaitResult::Message(msg) => {
                    match &msg {
                        URCMessages::Join(join) => {
                            #[cfg(feature = "debug")]
                            info!("Join URC");
                            match join {
                                JoinUrc::Failed => {
                                    #[cfg(feature = "debug")]
                                    info!("Failed to join");
                                }
                                JoinUrc::JoinedAlready => {
                                    #[cfg(feature = "debug")]
                                    info!("Already joined");
                                    self.join_status.join_status = JoinStatus::Success;
                                }
                                JoinUrc::Success(net_id, dev_addr) => {
                                    #[cfg(feature = "debug")]
                                    info!(
                                        "Joined network: net_id: {}, dev_addr: {}",
                                        net_id.as_str(),
                                        dev_addr.as_str()
                                    );
                                    self.join_status.join_status = JoinStatus::Success;
                                    self.join_status.net_id = Some(net_id.clone());
                                    self.join_status.dev_addr = Some(dev_addr.clone());
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    };
                    Ok(Some(msg))
                }
            }
        }
    }
}

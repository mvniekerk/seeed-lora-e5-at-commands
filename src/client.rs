#[cfg(feature = "async")]
pub mod asynch {
    use crate::general::responses::VerResponse;
    use crate::lora::urc::{JoinUrc, MessageReceived};
    use crate::urc::URCMessages;
    pub use atat::asynch::Client;
    use atat::{Error, UrcSubscription};
    #[cfg(feature = "debug")]
    use defmt::{error, info, warn};
    use embassy_sync::pubsub::WaitResult;
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
        //
        //     pub async fn service_urc(&mut self) -> Result<(), Error> {
        //         let mut msg_read = self.urc_subscription.try_next_message();
        //         if msg_read.is_none() {
        //             return Ok(());
        //         }
        //
        //         while msg_read.is_some() {
        //             let msg = msg_read.unwrap();
        //             match msg {
        //                 WaitResult::Lagged(amount) => {
        //                     #[cfg(feature = "debug")]
        //                     warn!("Missed URC messages {}", amount);
        //                     msg_read = self.urc_subscription.try_next_message();
        //                 }
        //                 WaitResult::Message(msg) => {
        //                     match &msg {
        //                         // Join
        //                         URCMessages::Join(join) => {
        //                             #[cfg(feature = "debug")]
        //                             info!("Join URC");
        //                             match join {
        //                                 JoinUrc::Start | JoinUrc::Normal => {
        //                                     msg_read = Some(self.urc_subscription.next_message().await);
        //                                 }
        //                                 JoinUrc::Failed => {
        //                                     #[cfg(feature = "debug")]
        //                                     info!("Failed to join");
        //                                     msg_read = None;
        //                                 }
        //                                 JoinUrc::JoinedAlready => {
        //                                     #[cfg(feature = "debug")]
        //                                     info!("Already joined");
        //                                     self.join_status.join_status = JoinStatus::Success;
        //                                     msg_read = None;
        //                                 }
        //                                 JoinUrc::Success(net_id, dev_addr) => {
        //                                     #[cfg(feature = "debug")]
        //                                     info!(
        //                                         "Joined network: net_id: {}, dev_addr: {}",
        //                                         net_id.as_str(),
        //                                         dev_addr.as_str()
        //                                     );
        //                                     self.join_status.join_status = JoinStatus::Success;
        //                                     self.join_status.net_id = Some(net_id.clone());
        //                                     self.join_status.dev_addr = Some(dev_addr.clone());
        //                                     msg_read = None;
        //                                 }
        //                                 _ => {
        //                                     #[cfg(feature = "debug")]
        //                                     info!("Unhandled Join URC");
        //                                     msg_read = self.urc_subscription.try_next_message()
        //                                 }
        //                             }
        //                         }
        //                         URCMessages::MessageReceived(rx) => {
        //                             #[cfg(feature = "debug")]
        //                             info!("Message RX URC");
        //                             match rx {
        //                                 MessageReceived::Payload(payload) => {
        //                                     self.message_receive.payload = payload.payload.clone();
        //                                     self.message_receive.port = payload.port;
        //                                     self.message_receive.length = payload.length;
        //                                     msg_read = Some(self.urc_subscription.next_message().await);
        //                                 }
        //                                 MessageReceived::RxWinRssiSnr(rxwin, rssi, snr) => {
        //                                     self.message_receive.rxwin = *rxwin;
        //                                     self.message_receive.rssi = *rssi;
        //                                     self.message_receive.snr = *snr;
        //                                     msg_read = Some(self.urc_subscription.next_message().await);
        //                                 }
        //                                 MessageReceived::Done => {
        //                                     msg_read = None;
        //                                 }
        //                             }
        //                         }
        //                         _ => {
        //                             #[cfg(feature = "debug")]
        //                             info!("Unhandled URC");
        //                             msg_read = self.urc_subscription.try_next_message()
        //                         }
        //                     };
        //                 }
        //             }
        //         }
        //         Ok(())
        //     }
    }
}

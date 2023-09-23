pub mod commands;
pub mod responses;
pub mod types;

#[cfg(feature = "async")]
pub mod asynch {
    use crate::client::asynch::SeeedLoraE5Client;
    use crate::general::commands::{FactoryReset, FirmwareVersion, Reset, VerifyComIsWorking};
    use crate::general::responses::VerResponse;
    use atat::asynch::AtatClient;
    use atat::Error;
    #[cfg(feature = "debug")]
    use defmt::error;
    use embedded_io_async::Write;

    impl<'a, W: Write, const INGRESS_BUF_SIZE: usize> SeeedLoraE5Client<'a, W, INGRESS_BUF_SIZE> {
        pub async fn verify_com_is_working(&mut self) -> Result<bool, Error> {
            let command = VerifyComIsWorking {};
            let response = self.client.send(&command).await?;
            Ok(response.is_ok())
        }

        pub async fn at_echo_on(&mut self) -> Result<bool, Error> {
            // Nop
            Ok(true)
        }

        pub async fn at_echo_set(&mut self, _on: bool) -> Result<bool, Error> {
            // Nop
            Ok(true)
        }

        pub async fn version(&mut self) -> Result<VerResponse, Error> {
            let command = FirmwareVersion {};
            let response = self.client.send(&command).await?;
            Ok(response)
        }

        pub async fn reset(&mut self) -> Result<(), Error> {
            let command = Reset {};
            let resp = self.client.send(&command).await;
            if let Err(e) = resp {
                #[cfg(feature = "debug")]
                error!("Error resetting Seeed LoRa-E5: {:?}", e);
                return Err(e);
            }
            Ok(())
        }

        pub async fn factory_reset(&mut self) -> Result<(), Error> {
            let command = FactoryReset {};
            let resp = self.client.send(&command).await;
            if let Err(e) = resp {
                #[cfg(feature = "debug")]
                error!("Error factory resetting Seeed LoRa-E5: {:?}", e);
                return Err(e);
            }
            Ok(())
        }
    }
}

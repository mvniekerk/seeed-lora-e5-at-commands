pub mod commands;
pub mod responses;
pub mod types;
pub mod urc;

#[cfg(feature = "async")]
pub mod asynch {
    use crate::client::asynch::{JoinStatus, SeeedLoraE5Client};
    use crate::lora::types::LoraJoinMode;
    use crate::lora::{
        commands,
        types::{LoraClass, LoraJoiningStatus, LoraRegion},
    };
    use crate::urc::{
        MessageStats, ReceivedMessage, LAST_LORA_MESSAGE_RECEIVED, LORA_JOIN_STATUS,
        LORA_MESSAGE_RECEIVED_COUNT, LORA_MESSAGE_RECEIVED_STATS,
    };
    use atat::asynch::AtatClient;
    use atat::Error;
    use embedded_io::asynch::Write;
    use heapless::String;
    use serde_at::HexStr;

    static mut CONFIRMED_SENDING: Option<bool> = Some(false);

    impl<'a, W: Write, const INGRESS_BUF_SIZE: usize> SeeedLoraE5Client<'a, W, INGRESS_BUF_SIZE> {
        pub async fn join_mode(&mut self) -> Result<LoraJoinMode, Error> {
            let command = commands::ModeGet {};
            let response = self.client.send(&command).await?;
            Ok(LoraJoinMode::from(response))
        }

        pub async fn join_mode_set(&mut self, mode: LoraJoinMode) -> Result<LoraJoinMode, Error> {
            let command = match mode {
                LoraJoinMode::Otaa => commands::ModeSet::otaa(),
                LoraJoinMode::Abp => commands::ModeSet::abp(),
                LoraJoinMode::Test => commands::ModeSet::test(),
                _ => return Err(Error::Error),
            };
            let response = self.client.send(&command).await?;
            Ok(response.mode())
        }

        pub async fn dev_eui(&mut self) -> Result<u64, Error> {
            let command = commands::DevEuiGet {};
            let response = self.client.send(&command).await?;
            Ok(response.dev_eui.val)
        }

        pub async fn dev_eui_set(&mut self, dev_eui: u64) -> Result<u64, Error> {
            let command = commands::DevEuiSet::dev_eui(dev_eui);
            let response = self.client.send(&command).await?;
            Ok(response.dev_eui.val)
        }

        pub async fn app_eui(&mut self) -> Result<u64, Error> {
            let command = commands::AppEuiGet {};
            let response = self.client.send(&command).await?;
            Ok(response.app_eui.val)
        }

        pub async fn app_eui_set(&mut self, app_eui: u64) -> Result<u64, Error> {
            let command = commands::AppEuiSet::app_eui(app_eui);
            let response = self.client.send(&command).await?;
            Ok(response.app_eui.val)
        }

        pub async fn app_key_set(&mut self, app_key: u128) -> Result<(), Error> {
            let command = commands::AppKeySet::app_key(app_key);
            self.client.send(&command).await?;
            Ok(())
        }

        pub async fn lora_region(&mut self) -> Result<LoraRegion, Error> {
            let command = commands::LoraDrGet {};
            let response = self.client.send(&command).await?;
            let s = response.rate.as_str();
            let s: String<24> = s.into();
            Ok(s.into())
        }

        pub async fn lora_region_set(&mut self, region: LoraRegion) -> Result<LoraRegion, Error> {
            let command = commands::DataRateSchemeSet::region(region);
            let response = self.client.send(&command).await?;
            let s = response.rate.as_str();
            let s: String<24> = s.into();
            Ok(s.into())
        }

        pub async fn lora_class(&mut self) -> Result<LoraClass, Error> {
            let command = commands::LoraClassGet {};
            let response = self.client.send(&command).await?;
            Ok(response.class.into())
        }

        pub async fn lora_class_set(&mut self, class: LoraClass) -> Result<LoraClass, Error> {
            let command = commands::LoraClassSet::class(class);
            let response = self.client.send(&command).await?;
            Ok(response.class.into())
        }

        pub async fn lora_join_otaa(&mut self) -> Result<LoraJoiningStatus, Error> {
            self.join_status.join_status = JoinStatus::Joining;
            LORA_JOIN_STATUS.signal(JoinStatus::Joining);
            let command = commands::LoraJoinOtaa {};
            let response = self
                .client
                .send(&command)
                .await
                .map_err(|e| {
                    LORA_JOIN_STATUS.signal(JoinStatus::NotJoined);
                    self.join_status.join_status = JoinStatus::NotJoined;
                    e
                })?
                .response;
            Ok(response.into())
        }

        pub async fn lora_join_status(&mut self) -> Result<JoinStatus, Error> {
            Ok(LORA_JOIN_STATUS
                .try_signaled_value()
                .unwrap_or(JoinStatus::NotJoined))
        }

        pub async fn lora_join_otaa_and_wait_for_result(&mut self) -> Result<JoinStatus, Error> {
            self.lora_join_otaa().await?;
            loop {
                let status = LORA_JOIN_STATUS.wait().await;
                if matches!(
                    status,
                    JoinStatus::Success | JoinStatus::Failure | JoinStatus::NotJoined
                ) {
                    return Ok(status);
                }
            }
        }

        pub async fn auto_join_set(
            &mut self,
            is_on: bool,
            interval: u32,
        ) -> Result<String<26>, Error> {
            let response = if is_on {
                let command = commands::LoraAutoJoinOtaaMode0 { interval };
                self.client.send(&command).await?
            } else {
                let command = commands::LoraAutoJoinOtaaDisable {};
                self.client.send(&command).await?
            };
            Ok(response.response)
        }

        pub async fn max_tx_len(&mut self) -> Result<u8, Error> {
            let command = commands::LoraMaxTxLengthGet::default();
            let response = self.client.send(&command).await?;
            Ok(response.max)
        }

        pub async fn confirm_send(&mut self) -> Result<bool, Error> {
            let confirmed_sending = unsafe { CONFIRMED_SENDING.unwrap() };
            Ok(confirmed_sending)
        }

        pub async fn confirm_send_set(&mut self, is_on: bool) -> Result<bool, Error> {
            unsafe {
                CONFIRMED_SENDING = Some(is_on);
            }
            Ok(is_on)
        }

        pub async fn send(
            &mut self,
            retransmission_times: u8,
            port: u8,
            data: &[u8],
        ) -> Result<(), Error> {
            let mut val = [0u8; 242];
            for (place, array) in val.iter_mut().zip(data.iter()) {
                *place = *array;
            }

            let message = HexStr {
                val,
                add_0x_with_encoding: false,
                hex_in_caps: false,
                delimiter_after_nibble_count: 0,
                delimiter: ' ',
                skip_last_0_values: true,
            };
            let port_set = commands::LoraPortSet { port };
            let _response = self.client.send(&port_set).await?;
            match self.confirm_send().await? {
                true => {
                    let retry = commands::RetrySet {
                        retry: retransmission_times,
                    };
                    let _response = self.client.send(&retry).await?;
                    let command = commands::MessageHexConfirmed { message };
                    let _response = self.client.send(&command).await?;
                    Ok(())
                }
                false => {
                    let repeat = commands::RepeatSet {
                        repeat: retransmission_times,
                    };
                    let _response = self.client.send(&repeat).await?;
                    let command = commands::MessageHexConfirmed { message };
                    let _response = self.client.send(&command).await?;
                    Ok(())
                }
            }
        }

        pub async fn receive(&mut self) -> Result<(ReceivedMessage, MessageStats), Error> {
            let value = LAST_LORA_MESSAGE_RECEIVED.wait().await;
            LAST_LORA_MESSAGE_RECEIVED.reset();
            let stats = LORA_MESSAGE_RECEIVED_STATS.wait().await;
            LORA_MESSAGE_RECEIVED_STATS.reset();
            Ok((value, stats))
        }

        pub async fn adr_set(&mut self, on: bool) -> Result<bool, Error> {
            let command = if on {
                commands::LoraAdrSet::on()
            } else {
                commands::LoraAdrSet::off()
            };
            let response = self.client.send(&command).await?;
            Ok(response.is_on())
        }

        pub async fn dr_set(&mut self, data_rate: u8) -> Result<u8, Error> {
            let command = commands::LoraDrSet::new(data_rate);
            let _response = self.client.send(&command).await?;
            Ok(data_rate)
        }

        pub async fn uplink_frame_count(&mut self) -> Result<u32, Error> {
            let command = commands::LoraUplinkDownlinkCounterGet {};
            let response = self.client.send(&command).await?;
            Ok(response.uplink())
        }

        pub async fn downlink_frame_count(&mut self) -> Result<u32, Error> {
            let command = commands::LoraUplinkDownlinkCounterGet {};
            let response = self.client.send(&command).await?;
            Ok(response.downlink())
        }

        pub async fn downlink_message_count(&self) -> Result<u32, Error> {
            Ok(LORA_MESSAGE_RECEIVED_COUNT
                .try_signaled_value()
                .unwrap_or_default())
        }
    }
}

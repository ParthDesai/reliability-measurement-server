use std::time::Instant;

use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use shared::{Challenge, Data, Message};
use warp::ws::WebSocket;

use crate::measurements::helpers::{
    verify_cpu_challenge_response, verify_network_challenge_response,
};
use crate::measurements::score::calculate_score;
use crate::types::{ClientData, Storage, WsMessage};
use crate::utils::send_client_msg_with_profiling;
use futures::stream::{SplitSink, SplitStream};
use rand::rngs::OsRng;
use rand::RngCore;
use shared::challenges::roundtrip::Roundtrip;
use shared::challenges::timelock::Timelock;

pub struct CPUChallengeConfiguration {
    pub squarings: u32,
    pub ideal_milliseconds: u128,
    pub max_milliseconds: u128,
}

pub struct NetworkChallengeConfiguration {
    pub data_size_kb: usize,
    pub ideal_milliseconds: u128,
    pub max_milliseconds: u128,
}

struct ClientChallenger {
    pub cpu_challenge_config: CPUChallengeConfiguration,
    pub network_challenge_config: NetworkChallengeConfiguration,
    pub number_of_cpu_challenge: usize,
    pub number_of_network_challenge: usize,
}

impl ClientChallenger {
    fn determine_score(&self, cpu_results: &Vec<u128>, network_results: &Vec<u128>) -> u128 {
        calculate_score(
            &self.cpu_challenge_config,
            cpu_results,
            &self.network_challenge_config,
            network_results,
        )
    }

    /// Performs cpu challenge as per the configuration and
    /// returns time elapsed
    async fn perform_cpu_challenge<RNG>(
        &self,
        rng: &mut RNG,
        client_id: u128,
        writer: &mut SplitSink<WebSocket, WsMessage>,
        reader: &mut SplitStream<WebSocket>,
    ) -> Result<u128>
    where
        RNG: RngCore,
    {
        let start = Instant::now();
        let (timelock, timelock_verifier) =
            Timelock::generate(rng, self.cpu_challenge_config.squarings);
        let time_passed = start.elapsed().as_millis();
        info!(
            "Internal: Generated CPU based puzzle in {}ms for client {:x}",
            time_passed, client_id
        );

        let challenge_msg = timelock.to_wire();
        let encoded_challenge_msg =
            Message::Challenge(Challenge::CPUChallenge(challenge_msg)).encode()?;

        let (client_response, time_elapsed) =
            send_client_msg_with_profiling(writer, reader, encoded_challenge_msg.as_slice(), false)
                .await?;

        if !verify_cpu_challenge_response(timelock_verifier, client_response) {
            info!(
                "Failed CPU measurements for client {:x}, time passed: {}ms",
                client_id, time_passed
            );
            writer
                .send(WsMessage::binary(
                    Message::Data(Data::Error("Failed CPU measurements".to_owned())).encode()?,
                ))
                .await?;
            return Err(anyhow!(format!(
                "CPU measurement failed for client {:x}",
                client_id
            )));
        } else {
            info!(
                "Successfully measured CPU power for client {:x}, time passed: {}ms",
                client_id, time_elapsed
            );
        }

        Ok(time_elapsed)
    }

    /// Performs network challenge as per the configuration and
    /// returns time elapsed
    async fn perform_network_challenge<RNG>(
        &self,
        rng: &mut RNG,
        client_id: u128,
        writer: &mut SplitSink<WebSocket, WsMessage>,
        reader: &mut SplitStream<WebSocket>,
    ) -> Result<u128>
    where
        RNG: RngCore,
    {
        let (roundtrip, roundtrip_verifier) =
            Roundtrip::generate(rng, self.network_challenge_config.data_size_kb);
        let encoded_challenge_msg =
            Message::Challenge(Challenge::NetworkChallenge(roundtrip.to_wire())).encode()?;

        let (client_response, time_elapsed) =
            send_client_msg_with_profiling(writer, reader, encoded_challenge_msg.as_slice(), true)
                .await?;

        if !verify_network_challenge_response(roundtrip_verifier, client_response) {
            info!(
                "Failed Network measurements for client {:x}, time passed: {}ms",
                client_id, time_elapsed
            );
            writer
                .send(WsMessage::binary(
                    Message::Data(Data::Error("Failed Network measurements".to_owned()))
                        .encode()?,
                ))
                .await?;
            return Err(anyhow!(format!(
                "Network measurement failed for client {:x}",
                client_id
            )));
        } else {
            info!(
                "Successfully measured Network bandwidth for client {:x}, time passed: {}ms",
                client_id, time_elapsed
            );
        }

        Ok(time_elapsed)
    }

    pub async fn challenge_client(
        &self,
        ws: WebSocket,
        storage: Storage,
        client_id: u128,
    ) -> Result<()> {
        let mut rng = OsRng::default();
        let (mut writer, mut reader) = ws.split();
        let mut cpu_results = vec![0u128; self.number_of_cpu_challenge];
        let mut network_results = vec![0u128; self.number_of_network_challenge];

        info!(
            "Internal: Starting measurements for client {:x}\n",
            client_id
        );

        info!(
            "Internal: Starting CPU measurements for client {:x}\n",
            client_id
        );

        for i in 0..self.number_of_cpu_challenge {
            cpu_results[i] = self
                .perform_cpu_challenge(&mut rng, client_id, &mut writer, &mut reader)
                .await?;
        }

        info!(
            "Internal: Starting Network measurements for client {:x}",
            client_id
        );

        for i in 0..self.number_of_network_challenge {
            network_results[i] = self
                .perform_network_challenge(&mut rng, client_id, &mut writer, &mut reader)
                .await?;
        }

        let client_score = self.determine_score(&cpu_results, &network_results);
        info!("Score for client {:x} is {}", client_id, client_score);
        storage.write().await.insert(
            client_id,
            ClientData {
                score: client_score,
                cpu_challenge_timings_in_milis: cpu_results,
                network_challenge_timings_in_milis: network_results,
            },
        );

        writer
            .send(WsMessage::binary(
                Message::Data(Data::Info(
                    format!("My score is: {}", client_score).to_owned(),
                ))
                .encode()?,
            ))
            .await?;

        Ok(())
    }
}

pub(crate) async fn perform_all(ws: WebSocket, storage: Storage, client_id: u128) -> Result<()> {
    let challenger = ClientChallenger {
        cpu_challenge_config: CPUChallengeConfiguration {
            squarings: 200000,
            ideal_milliseconds: 4500,
            max_milliseconds: 120000,
        },
        network_challenge_config: NetworkChallengeConfiguration {
            data_size_kb: 1024,
            ideal_milliseconds: 200,
            max_milliseconds: 25000,
        },
        number_of_cpu_challenge: 5,
        number_of_network_challenge: 10,
    };

    challenger.challenge_client(ws, storage, client_id).await
}

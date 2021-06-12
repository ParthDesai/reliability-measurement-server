use shared::{Message, Response};

use num_bigint::BigUint;
use shared::challenges::roundtrip::RoundtripVerifier;
use shared::challenges::timelock::TimelockVerifier;

pub(crate) fn verify_network_challenge_response(
    roundtrip_verifier: RoundtripVerifier,
    response: Message,
) -> bool {
    match response {
        Message::Response(response) => match response {
            Response::NetworkChallengeResponse(serialized_answer) => {
                roundtrip_verifier.verify(serialized_answer)
            }
            _ => false,
        },
        _ => false,
    }
}

pub(crate) fn verify_cpu_challenge_response(
    timelock_verifier: TimelockVerifier,
    response: Message,
) -> bool {
    match response {
        Message::Response(response) => match response {
            Response::CPUChallengeResponse(serialized_answer) => {
                let client_answer = BigUint::from_bytes_be(serialized_answer.as_slice());
                timelock_verifier.verify(client_answer)
            }
            _ => false,
        },
        _ => false,
    }
}

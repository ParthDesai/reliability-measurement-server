use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub(crate) struct ClientData {
    pub(crate) score: u128,
    pub(crate) cpu_challenge_timings_in_milis: Vec<u128>,
    pub(crate) network_challenge_timings_in_milis: Vec<u128>,
}

pub(crate) type Storage = Arc<RwLock<HashMap<u128, ClientData>>>;

pub(crate) type WsMessage = warp::ws::Message;

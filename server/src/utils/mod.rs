#[macro_export]
macro_rules! err {
    ($($t:tt)*) => (Err(anyhow::anyhow!($($t)*)))
}

pub mod network;

pub(crate) use network::send_client_msg_with_profiling;

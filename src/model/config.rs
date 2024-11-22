use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ConfigTable {
    #[serde(rename = "Config")]
    pub config: Config,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub retry_interval: u64, // seconds
    pub max_batch_size: u8,  // number
    pub max_concurrency: u8, // number
}

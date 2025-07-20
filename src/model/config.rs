use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ConfigTable {
    #[serde(rename = "Config")]
    pub config: Config,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub internal_timestamp: i64,    // mini second
    pub default_wakeup_time: i64,   // second
    pub max_concurrency: u8,        // number
    pub max_file_operations: usize, // number
}

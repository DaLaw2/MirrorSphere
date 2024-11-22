use crate::utils::logging::Logging;
use tracing::info;
use crate::core::config_manager::ConfigManager;
use crate::core::database_manager::DatabaseManager;
use crate::utils::log_entry::system::SystemEntry;

pub struct System {}

impl System {
    pub async fn initialize() {
        Logging::initialize().await;
        info!("{}", SystemEntry::Initializing);
        ConfigManager::initialization().await;
        DatabaseManager::initialization().await;

        info!("{}", SystemEntry::InitializeComplete);
    }

    pub async fn run() {
        info!("{}", SystemEntry::Online);

    }

    pub async fn terminate() {
        info!("{}", SystemEntry::Terminating);

        info!("{}", SystemEntry::TerminateComplete);
    }
}

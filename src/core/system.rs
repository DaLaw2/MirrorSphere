use crate::core::app_config::AppConfig;
use crate::core::database_manager::DatabaseManager;
use crate::core::engine::Engine;
use crate::core::io_manager::IOManager;
use crate::platform::elevate::elevate;
use crate::utils::log_entry::system::SystemEntry;
use crate::utils::logging::Logging;
use crate::utils::privilege::elevate;
use privilege::user::privileged;
use tracing::info;

pub struct System {}

impl System {
    pub async fn initialize() {
        Logging::initialize().await;
        info!("{}", SystemEntry::Initializing);
        if !privileged() {
            info!("{}", SystemEntry::ReRunAsAdmin);
            elevate().map_err(SystemEntry::RunAsAdminFailed).unwrap();
        }
        AppConfig::initialization().await;
        Engine::initialize().await;
        IOManager::initialize().await;
        DatabaseManager::initialization().await;
        info!("{}", SystemEntry::InitializeComplete);
    }

    pub async fn run() {
        Engine::run().await;
        info!("{}", SystemEntry::Online);
    }

    pub async fn terminate() {
        info!("{}", SystemEntry::Terminating);
        Engine::terminate().await;
        DatabaseManager::terminate().await;
        info!("{}", SystemEntry::TerminateComplete);
    }
}

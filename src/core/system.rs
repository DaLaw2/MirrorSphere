use crate::core::app_config::AppConfig;
use crate::core::database_manager::DatabaseManager;
use crate::core::engine::Engine;
use crate::core::io_manager::IOManager;
use crate::model::error::system::SystemError;
use crate::model::log::system::SystemLog;
use crate::platform::elevate::elevate;
use crate::utils::logging::Logging;
use privilege::user::privileged;
use std::process;

pub struct System;

impl System {
    pub async fn initialize() {
        Logging::initialize().await;
        SystemLog::Initializing.log();
        if !privileged() {
            SystemLog::ReRunAsAdmin.log();
            elevate()
                .map_err(|_| SystemError::RunAsAdminFailed)
                .unwrap();
            process::exit(0);
        }
        AppConfig::initialization().await;
        Engine::initialize().await;
        IOManager::initialize().await;
        DatabaseManager::initialization().await;
        SystemLog::InitializeComplete.log();
    }

    pub async fn run() {
        SystemLog::Online.log();
    }

    pub async fn terminate() {
        SystemLog::Terminating.log();
        Engine::terminate().await;
        DatabaseManager::terminate().await;
        SystemLog::TerminateComplete.log();
    }
}

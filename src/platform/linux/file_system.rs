use crate::interface::file_system::FileSystemTrait;
use crate::platform::attributes::{AdvancedAttributes, Attributes, PermissionAttributes};
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;

pub struct FileSystem {
    semaphore: Arc<Semaphore>,
}

#[async_trait]
impl FileSystemTrait for FileSystem {
    fn new(semaphore: Arc<Semaphore>) -> Self {
        FileSystem { semaphore }
    }

    fn semaphore(&self) -> Arc<Semaphore> {
        self.semaphore.clone()
    }

    async fn get_attributes(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<Attributes> {
        todo!()
    }

    async fn get_advanced_attributes(
        &self,
        task_id: Uuid,
        path: PathBuf,
    ) -> anyhow::Result<AdvancedAttributes> {
        todo!()
    }

    async fn set_attributes(
        &self,
        task_id: Uuid,
        path: PathBuf,
        attributes: Attributes,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn set_advanced_attributes(
        &self,
        task_id: Uuid,
        path: PathBuf,
        attributes: AdvancedAttributes,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn compare_attributes(
        &self,
        task_id: Uuid,
        source: PathBuf,
        destination: PathBuf,
    ) -> anyhow::Result<bool> {
        todo!()
    }

    async fn compare_advanced_attributes(
        &self,
        task_id: Uuid,
        source: PathBuf,
        destination: PathBuf,
    ) -> anyhow::Result<bool> {
        todo!()
    }

    async fn get_permission(
        &self,
        task_id: Uuid,
        path: PathBuf,
    ) -> anyhow::Result<PermissionAttributes> {
        todo!()
    }

    async fn set_permission(
        &self,
        task_id: Uuid,
        permission: PermissionAttributes,
    ) -> anyhow::Result<()> {
        todo!()
    }
}

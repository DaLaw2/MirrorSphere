use crate::interface::file_system::FileSystemTrait;
use crate::platform::attributes::{Attributes, Permissions};
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;

pub struct FileSystem {
    semaphore: Arc<Semaphore>,
}

#[async_trait]
#[cfg(target_os = "linux")]
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

    async fn set_attributes(
        &self,
        task_id: Uuid,
        path: PathBuf,
        attributes: Attributes,
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

    async fn get_permission(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<Permissions> {
        todo!()
    }

    async fn set_permission(
        &self,
        task_id: Uuid,
        path: PathBuf,
        permission: Permissions,
    ) -> anyhow::Result<()> {
        todo!()
    }
}

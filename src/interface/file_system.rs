use crate::core::event_system::event_bus::EventBus;
use crate::model::event::io_event::{IOEvent, IOType};
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Semaphore;
use tokio_stream::wrappers::ReadDirStream;
use tokio_stream::StreamExt;
use uuid::Uuid;

#[async_trait]
pub trait FileSystemTrait {
    fn new(semaphore: Arc<Semaphore>) -> Self;

    fn semaphore(&self) -> Arc<Semaphore>;

    async fn list_directory(&self, path: PathBuf) -> anyhow::Result<Vec<PathBuf>> {
        let semaphore = self.semaphore();
        let _permit = semaphore.acquire_owned().await?;
        let mut result = Vec::new();
        let reader = fs::read_dir(path).await?;
        let mut entries = ReadDirStream::new(reader);
        while let Some(entry) = entries.next().await {
            result.push(entry?.path());
        }
        Ok(result)
    }

    async fn create_directory(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore.acquire_owned().await?;
        fs::create_dir_all(&path).await?;
        let io_event = IOEvent {
            task_id,
            io_type: IOType::CreateDirectory,
            source: None,
            destination,
        };
        EventBus::publish(io_event).await?;
        Ok(())
    }

    async fn copy_file(
        &self,
        task_id: Uuid,
        source: PathBuf,
        destination: PathBuf,
    ) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore.acquire_owned().await?;
        fs::copy(&source, &destination).await?;
        let io_event = IOEvent {
            task_id,
            io_type: IOType::CopyFile,
            source: Some(source),
            destination,
        };
        EventBus::publish(io_event).await?;
        Ok(())
    }

    async fn delete_directory(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore.acquire_owned().await?;
        fs::remove_dir_all(&path).await?;
        let io_event = IOEvent {
            task_id,
            io_type: IOType::DeleteDirectory,
            source: None,
            destination,
        };
        EventBus::publish(io_event).await?;
        Ok(())
    }

    async fn delete_file(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore.acquire_owned().await?;
        fs::remove_file(path).await?;
        let io_event = IOEvent {
            task_id,
            io_type: IOType::DeleteFile,
            source: None,
            destination,
        };
        EventBus::publish(io_event).await?;
        Ok(())
    }
}

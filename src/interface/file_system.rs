use crate::core::event_system::event_bus::EventBus;
use crate::model::event::io::directory::CreateDirectoryEvent;
use crate::model::event::io::directory::DeleteDirectoryEvent;
use crate::model::event::io::directory::ListDirectoryEvent;
use crate::model::event::io::file::CopyFileEvent;
use crate::model::event::io::file::DeleteFileEvent;
use crate::utils::log_entry::io::IOEntry;
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

    async fn list_directory(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<Vec<PathBuf>> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOEntry::SemaphoreClosed)?;
        let mut result = Vec::new();
        let reader = fs::read_dir(&path).await?;
        let mut entries = ReadDirStream::new(reader);
        while let Some(entry) = entries.next().await {
            result.push(entry?.path());
        }
        let event = ListDirectoryEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(result)
    }

    async fn create_directory(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOEntry::SemaphoreClosed)?;
        fs::create_dir_all(&path).await?;
        let event = CreateDirectoryEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(())
    }

    async fn copy_file(
        &self,
        task_id: Uuid,
        source: PathBuf,
        destination: PathBuf,
    ) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOEntry::SemaphoreClosed)?;
        fs::copy(&source, &destination).await?;
        let event = CopyFileEvent {
            task_id,
            source,
            destination,
        };
        EventBus::publish(event).await?;
        Ok(())
    }

    async fn delete_directory(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOEntry::SemaphoreClosed)?;
        fs::remove_dir_all(&path).await?;
        let event = DeleteDirectoryEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(())
    }

    async fn delete_file(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOEntry::SemaphoreClosed)?;
        fs::remove_file(&path).await?;
        let event = DeleteFileEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(())
    }
}

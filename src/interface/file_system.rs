use crate::core::event_system::event_bus::EventBus;
use crate::model::event::io::directory::*;
use crate::model::event::io::file::*;
use crate::model::event::io::hash::*;
use crate::model::task::HashType;
use crate::platform::attributes::*;
use crate::utils::file_hash::*;
use crate::utils::log_entry::io::IOEntry;
use crate::utils::log_entry::system::SystemEntry;
use async_trait::async_trait;
use digest::Digest;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Semaphore;
use tokio::task::spawn_blocking;
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
        let reader = fs::read_dir(&path)
            .await
            .map_err(|_| IOEntry::ReadDirectoryFailed)?;
        let mut entries = ReadDirStream::new(reader);
        while let Some(entry) = entries.next().await {
            let path = entry.map_err(|_| IOEntry::ReadFileFailed)?.path();
            result.push(path);
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
        fs::create_dir_all(&path)
            .await
            .map_err(|_| IOEntry::CreateDirectoryFailed)?;
        let event = CreateDirectoryEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(())
    }

    async fn delete_directory(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOEntry::SemaphoreClosed)?;
        fs::remove_dir_all(&path)
            .await
            .map_err(|_| IOEntry::DeleteDirectoryFailed)?;
        let event = DeleteDirectoryEvent { task_id, path };
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
        fs::copy(&source, &destination)
            .await
            .map_err(|_| IOEntry::CopyFileFailed)?;
        let event = CopyFileEvent {
            task_id,
            source,
            destination,
        };
        EventBus::publish(event).await?;
        Ok(())
    }

    async fn delete_file(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOEntry::SemaphoreClosed)?;
        fs::remove_file(&path)
            .await
            .map_err(|_| IOEntry::DeleteFileFailed)?;
        let event = DeleteFileEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(())
    }

    async fn get_attributes(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<Attributes>;

    async fn get_advanced_attributes(
        &self,
        task_id: Uuid,
        path: PathBuf,
    ) -> anyhow::Result<AdvancedAttributes>;

    async fn set_attributes(
        &self,
        task_id: Uuid,
        path: PathBuf,
        attributes: Attributes,
    ) -> anyhow::Result<()>;

    async fn set_advanced_attributes(
        &self,
        task_id: Uuid,
        path: PathBuf,
        attributes: AdvancedAttributes,
    ) -> anyhow::Result<()>;

    async fn compare_attributes(
        &self,
        task_id: Uuid,
        source: PathBuf,
        destination: PathBuf,
    ) -> anyhow::Result<bool>;

    async fn compare_advanced_attributes(
        &self,
        task_id: Uuid,
        source: PathBuf,
        destination: PathBuf,
    ) -> anyhow::Result<bool>;

    async fn get_permission(
        &self,
        task_id: Uuid,
        path: PathBuf,
    ) -> anyhow::Result<PermissionAttributes>;

    async fn set_permission(
        &self,
        task_id: Uuid,
        permission: PermissionAttributes,
    ) -> anyhow::Result<()>;

    async fn standard_compare(
        &self,
        task_id: Uuid,
        source: PathBuf,
        destination: PathBuf,
        advanced_attributes: bool,
    ) -> anyhow::Result<bool> {
        let compare_result = if advanced_attributes {
            self.compare_attributes(task_id, source, destination).await?
        } else {
            self.compare_advanced_attributes(task_id, source, destination).await?
        };
        if !compare_result {
            return Ok(false);
        }

        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOEntry::SemaphoreClosed)?;

        let source_metadata = fs::metadata(&source)
            .await
            .map_err(|_| IOEntry::GetMetadataFailed)?;
        let destination_metadata = fs::metadata(&destination)
            .await
            .map_err(|_| IOEntry::GetMetadataFailed)?;

        if source_metadata.len() != destination_metadata.len() {
            return Ok(false);
        }
        let source_modified = source_metadata
            .modified()
            .map_err(|_| IOEntry::GetMetadataFailed)?;
        let destination_modified = destination_metadata
            .modified()
            .map_err(|_| IOEntry::GetMetadataFailed)?;
        if source_modified != destination_modified {
            return Ok(false);
        }

        Ok(true)
    }

    async fn thorough_compare(
        &self,
        task_id: Uuid,
        source: PathBuf,
        destination: PathBuf,
        hash_type: HashType,
        advanced_attributes: bool,
    ) -> anyhow::Result<bool> {
        if !self
            .standard_compare(
                task_id,
                source.clone(),
                destination.clone(),
                advanced_attributes,
            )
            .await?
        {
            return Ok(false);
        }
        let source_file_hash = self.calculate_hash(task_id, source, hash_type).await?;
        let destination_file_hash = self.calculate_hash(task_id, destination, hash_type).await?;

        Ok(source_file_hash == destination_file_hash)
    }

    async fn calculate_hash(
        &self,
        task_id: Uuid,
        path: PathBuf,
        hash_type: HashType,
    ) -> anyhow::Result<Vec<u8>> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOEntry::SemaphoreClosed)?;

        let hash = spawn_blocking(move || {
            let file = std::fs::File::open(&path)?;
            match hash_type {
                HashType::MD5 => md5(file),
                HashType::SHA3 => sha3(file),
                HashType::SHA256 => sha256(file),
                HashType::BLAKE2B => blake2b(file),
                HashType::BLAKE2S => blake2s(file),
                HashType::BLAKE3 => blake3(file),
            }
        })
        .await
        .map_err(|_| SystemEntry::ThreadPanic)??;

        let event = CalculateHashEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(hash)
    }
}

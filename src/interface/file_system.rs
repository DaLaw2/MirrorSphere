use crate::core::event_system::event_bus::EventBus;
use crate::model::error::io::IOError;
use crate::model::error::system::SystemError;
use crate::model::event::io::attributes::GetAttributesEvent;
use crate::model::event::io::directory::*;
use crate::model::event::io::file::*;
use crate::model::event::io::hash::*;
use crate::model::task::HashType;
use crate::platform::attributes::*;
use crate::utils::file_hash::*;
use async_trait::async_trait;
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

    async fn is_symlink(&self, path: PathBuf) -> anyhow::Result<bool> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOError::SemaphoreClosed)?;

        let symlink_metadata = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|_| IOError::GetMetadataFailed)?;

        Ok(symlink_metadata.file_type().is_symlink())
    }

    async fn list_directory(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<Vec<PathBuf>> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOError::SemaphoreClosed)?;
        let mut result = Vec::new();
        let reader = fs::read_dir(&path)
            .await
            .map_err(|_| IOError::ReadDirectoryFailed)?;
        let mut entries = ReadDirStream::new(reader);
        while let Some(entry) = entries.next().await {
            let path = entry.map_err(|_| IOError::ReadFileFailed)?.path();
            result.push(path);
        }
        Ok(result)
    }

    async fn create_directory(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOError::SemaphoreClosed)?;
        fs::create_dir_all(&path)
            .await
            .map_err(|_| IOError::CreateDirectoryFailed)?;
        let event = CreateDirectoryEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(())
    }

    async fn delete_directory(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOError::SemaphoreClosed)?;
        fs::remove_dir_all(&path)
            .await
            .map_err(|_| IOError::DeleteDirectoryFailed)?;
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
            .map_err(|_| IOError::SemaphoreClosed)?;
        fs::copy(&source, &destination)
            .await
            .map_err(|_| IOError::CopyFileFailed)?;
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
            .map_err(|_| IOError::SemaphoreClosed)?;
        fs::remove_file(&path)
            .await
            .map_err(|_| IOError::DeleteFileFailed)?;
        let event = DeleteFileEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(())
    }

    async fn get_attributes(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<Attributes>;

    async fn set_attributes(
        &self,
        task_id: Uuid,
        path: PathBuf,
        attributes: Attributes,
    ) -> anyhow::Result<()>;

    async fn copy_attributes(
        &self,
        task_id: Uuid,
        source: PathBuf,
        destination: PathBuf,
    ) -> anyhow::Result<()> {
        let source_attributes = self.get_attributes(task_id, source.clone()).await?;
        self.set_attributes(task_id, destination, source_attributes)
            .await?;
        Ok(())
    }

    async fn compare_attributes(
        &self,
        task_id: Uuid,
        source: PathBuf,
        destination: PathBuf,
    ) -> anyhow::Result<bool> {
        let source_attributes = self.get_attributes(task_id, source.clone()).await?;
        let destination_attributes = self.get_attributes(task_id, destination.clone()).await?;
        Ok(source_attributes == destination_attributes)
    }

    async fn get_permission(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<Permissions>;

    async fn set_permission(
        &self,
        task_id: Uuid,
        path: PathBuf,
        permissions: Permissions,
    ) -> anyhow::Result<()>;

    async fn copy_permission(
        &self,
        task_id: Uuid,
        source: PathBuf,
        destination: PathBuf,
    ) -> anyhow::Result<()> {
        let source_permissions = self.get_permission(task_id, source.clone()).await?;
        self.set_permission(task_id, destination, source_permissions)
            .await?;
        Ok(())
    }

    async fn standard_compare(
        &self,
        task_id: Uuid,
        source: PathBuf,
        destination: PathBuf,
    ) -> anyhow::Result<bool> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOError::SemaphoreClosed)?;

        let source_metadata = fs::metadata(&source)
            .await
            .map_err(|_| IOError::GetMetadataFailed)?;
        let destination_metadata = fs::metadata(&destination)
            .await
            .map_err(|_| IOError::GetMetadataFailed)?;

        if source_metadata.len() != destination_metadata.len() {
            return Ok(false);
        }
        let source_modified = source_metadata
            .modified()
            .map_err(|_| IOError::GetMetadataFailed)?;
        let destination_modified = destination_metadata
            .modified()
            .map_err(|_| IOError::GetMetadataFailed)?;
        if source_modified != destination_modified {
            return Ok(false);
        }

        let event = GetAttributesEvent {
            task_id,
            path: source,
        };
        EventBus::publish(event).await?;
        let event = GetAttributesEvent {
            task_id,
            path: destination,
        };
        EventBus::publish(event).await?;
        Ok(true)
    }

    async fn advance_compare(
        &self,
        task_id: Uuid,
        source: PathBuf,
        destination: PathBuf,
    ) -> anyhow::Result<bool> {
        if !self
            .standard_compare(task_id, source.clone(), destination.clone())
            .await?
        {
            return Ok(false);
        }

        if !self
            .compare_attributes(task_id, source.clone(), destination.clone())
            .await?
        {
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
    ) -> anyhow::Result<bool> {
        if !self
            .advance_compare(task_id, source.clone(), destination.clone())
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
            .map_err(|_| IOError::SemaphoreClosed)?;

        let path_clone = path.clone();
        let hash = spawn_blocking(move || {
            let path = path_clone;
            let file = std::fs::File::open(path)?;
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
        .map_err(|_| SystemError::ThreadPanic)??;

        let event = CalculateHashEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(hash)
    }
}

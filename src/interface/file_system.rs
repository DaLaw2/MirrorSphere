use crate::model::error::io::IOError;
use crate::model::error::system::SystemError;
use crate::model::task::HashType;
use crate::platform::attributes::*;
use crate::platform::raii_guard::*;
use crate::utils::file_hash::*;
use async_trait::async_trait;
use fs4::tokio::AsyncFileExt;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Semaphore;
use tokio::task::spawn_blocking;
use tokio_stream::wrappers::ReadDirStream;
use tokio_stream::StreamExt;

#[async_trait]
pub trait FileSystemTrait {
    fn new(semaphore: Arc<Semaphore>) -> Self;

    fn semaphore(&self) -> Arc<Semaphore>;

    async fn is_symlink(&self, path: &PathBuf) -> anyhow::Result<bool> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOError::SemaphoreClosed)?;

        let symlink_metadata = tokio::fs::symlink_metadata(path)
            .await
            .map_err(|_| IOError::GetMetadataFailed { path: path.clone() })?;

        Ok(symlink_metadata.file_type().is_symlink())
    }

    async fn list_directory(&self, path: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOError::SemaphoreClosed)?;

        let mut result = Vec::new();
        let reader = fs::read_dir(path)
            .await
            .map_err(|_| IOError::ReadDirectoryFailed { path: path.clone() })?;
        let mut entries = ReadDirStream::new(reader);
        while let Some(entry) = entries.next().await {
            let path = entry
                .map_err(|_| IOError::ReadDirectoryFailed { path: path.clone() })?
                .path();
            result.push(path);
        }
        Ok(result)
    }

    async fn create_directory(&self, path: &PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOError::SemaphoreClosed)?;

        fs::create_dir_all(path)
            .await
            .map_err(|_| IOError::CreateDirectoryFailed { path: path.clone() })?;
        Ok(())
    }

    async fn delete_directory(&self, path: &PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOError::SemaphoreClosed)?;

        fs::remove_dir_all(path)
            .await
            .map_err(|_| IOError::DeleteDirectoryFailed { path: path.clone() })?;
        Ok(())
    }

    async fn copy_file(&self, source: &PathBuf, destination: &PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOError::SemaphoreClosed)?;

        fs::copy(source, destination)
            .await
            .map_err(|_| IOError::CopyFileFailed {
                src: source.clone(),
                dst: destination.clone(),
            })?;
        Ok(())
    }

    async fn delete_file(&self, path: &PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOError::SemaphoreClosed)?;

        fs::remove_file(path)
            .await
            .map_err(|_| IOError::DeleteFileFailed { path: path.clone() })?;
        Ok(())
    }

    async fn get_attributes(&self, path: &PathBuf) -> anyhow::Result<Attributes>;

    async fn set_attributes(&self, path: &PathBuf, attributes: Attributes) -> anyhow::Result<()>;

    async fn copy_attributes(&self, source: &PathBuf, destination: &PathBuf) -> anyhow::Result<()> {
        let source_attributes = self.get_attributes(source).await?;
        self.set_attributes(destination, source_attributes).await?;
        Ok(())
    }

    async fn compare_attributes(
        &self,
        source: &PathBuf,
        destination: &PathBuf,
    ) -> anyhow::Result<bool> {
        let source_attributes = self.get_attributes(source).await?;
        let destination_attributes = self.get_attributes(destination).await?;
        Ok(source_attributes == destination_attributes)
    }

    async fn get_permission(&self, path: &PathBuf) -> anyhow::Result<Permissions>;

    async fn set_permission(&self, path: &PathBuf, permissions: Permissions) -> anyhow::Result<()>;

    async fn copy_permission(&self, source: &PathBuf, destination: &PathBuf) -> anyhow::Result<()> {
        let source_permissions = self.get_permission(source).await?;
        self.set_permission(destination, source_permissions).await?;
        Ok(())
    }

    async fn acquire_file_lock(&self, path: &PathBuf) -> anyhow::Result<FileLockGuard> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOError::SemaphoreClosed)?;

        fs::File::open(path)
            .await
            .map_err(|_| IOError::ReadFileFailed { path: path.clone() })?
            .try_lock_exclusive()
            .map_err(|_| IOError::LockFileFailed { path: path.clone() })?;

        let file_lock = FileLockGuard::new(path.clone());

        Ok(file_lock)
    }

    async fn calculate_hash(&self, path: &PathBuf, hash_type: HashType) -> anyhow::Result<Vec<u8>> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOError::SemaphoreClosed)?;

        let path_clone = path.clone();
        let hash = spawn_blocking(move || {
            let path = path_clone;
            match hash_type {
                HashType::MD5 => md5(path),
                HashType::SHA3 => sha3(path),
                HashType::SHA256 => sha256(path),
                HashType::BLAKE2B => blake2b(path),
                HashType::BLAKE2S => blake2s(path),
                HashType::BLAKE3 => blake3(path),
            }
        })
        .await
        .map_err(|_| SystemError::ThreadPanic)??;
        Ok(hash)
    }

    async fn standard_compare(
        &self,
        source: &PathBuf,
        destination: &PathBuf,
    ) -> anyhow::Result<bool> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOError::SemaphoreClosed)?;

        let source_metadata =
            fs::metadata(source)
                .await
                .map_err(|_| IOError::GetMetadataFailed {
                    path: source.clone(),
                })?;
        let destination_metadata =
            fs::metadata(destination)
                .await
                .map_err(|_| IOError::GetMetadataFailed {
                    path: destination.clone(),
                })?;

        if source_metadata.len() != destination_metadata.len() {
            return Ok(false);
        }
        let source_modified =
            source_metadata
                .modified()
                .map_err(|_| IOError::GetMetadataFailed {
                    path: source.clone(),
                })?;
        let destination_modified =
            destination_metadata
                .modified()
                .map_err(|_| IOError::GetMetadataFailed {
                    path: destination.clone(),
                })?;
        if source_modified != destination_modified {
            return Ok(false);
        }
        Ok(true)
    }

    async fn advance_compare(
        &self,
        source: &PathBuf,
        destination: &PathBuf,
    ) -> anyhow::Result<bool> {
        if !self.standard_compare(source, destination).await? {
            return Ok(false);
        }

        if !self.compare_attributes(source, destination).await? {
            return Ok(false);
        }

        Ok(true)
    }

    async fn thorough_compare(
        &self,
        source: &PathBuf,
        destination: &PathBuf,
        hash_type: HashType,
    ) -> anyhow::Result<bool> {
        if !self.advance_compare(source, destination).await? {
            return Ok(false);
        }
        let source_file_hash = self.calculate_hash(source, hash_type).await?;
        let destination_file_hash = self.calculate_hash(destination, hash_type).await?;

        Ok(source_file_hash == destination_file_hash)
    }
}

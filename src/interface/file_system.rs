use crate::model::error::io::IOError;
use crate::model::error::system::SystemError;
use crate::model::error::Error;
use crate::model::backup::backup_execution::HashType;
use crate::platform::attributes::*;
use crate::utils::file_hash::*;
use crate::utils::file_lock::FileLock;
use async_trait::async_trait;
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

    async fn is_symlink(&self, path: &PathBuf) -> Result<bool, Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|err| IOError::SemaphoreClosed(err))?;

        let symlink_metadata = tokio::fs::symlink_metadata(path)
            .await
            .map_err(|err| IOError::GetMetadataFailed(path.clone(), err))?;

        Ok(symlink_metadata.file_type().is_symlink())
    }

    async fn create_symlink(&self, target: &PathBuf, link_path: &PathBuf) -> Result<(), Error>;

    async fn copy_symlink(
        &self,
        source_link: &PathBuf,
        destination_link: &PathBuf,
    ) -> Result<(), Error>;

    async fn list_directory(&self, path: &PathBuf) -> Result<Vec<PathBuf>, Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|err| IOError::SemaphoreClosed(err))?;

        let mut result = Vec::new();
        let reader = fs::read_dir(path)
            .await
            .map_err(|err| IOError::ReadDirectoryFailed(path.clone(), err))?;
        let mut entries = ReadDirStream::new(reader);
        while let Some(entry) = entries.next().await {
            let path = entry
                .map_err(|err| IOError::ReadDirectoryFailed(path.clone(), err))?
                .path();
            result.push(path);
        }
        Ok(result)
    }

    async fn create_directory(&self, path: &PathBuf) -> Result<(), Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|err| IOError::SemaphoreClosed(err))?;

        fs::create_dir_all(path)
            .await
            .map_err(|err| IOError::CreateDirectoryFailed(path.clone(), err))?;
        Ok(())
    }

    async fn delete_directory(&self, path: &PathBuf) -> Result<(), Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|err| IOError::SemaphoreClosed(err))?;

        fs::remove_dir_all(path)
            .await
            .map_err(|err| IOError::DeleteDirectoryFailed(path.clone(), err))?;
        Ok(())
    }

    async fn copy_file(&self, source: &PathBuf, destination: &PathBuf) -> Result<(), Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|err| IOError::SemaphoreClosed(err))?;

        fs::copy(source, destination)
            .await
            .map_err(|err| IOError::CopyFileFailed(source.clone(), destination.clone(), err))?;
        Ok(())
    }

    async fn delete_file(&self, path: &PathBuf) -> Result<(), Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|err| IOError::SemaphoreClosed(err))?;

        fs::remove_file(path)
            .await
            .map_err(|err| IOError::DeleteFileFailed(path.clone(), err))?;
        Ok(())
    }

    async fn get_attributes(&self, path: &PathBuf) -> Result<Attributes, Error>;

    async fn set_attributes(&self, path: &PathBuf, attributes: Attributes) -> Result<(), Error>;

    async fn copy_attributes(&self, source: &PathBuf, destination: &PathBuf) -> Result<(), Error> {
        let source_attributes = self.get_attributes(source).await?;
        self.set_attributes(destination, source_attributes).await?;
        Ok(())
    }

    async fn compare_attributes(
        &self,
        source: &PathBuf,
        destination: &PathBuf,
    ) -> Result<bool, Error> {
        let source_attributes = self.get_attributes(source).await?;
        let destination_attributes = self.get_attributes(destination).await?;
        Ok(source_attributes == destination_attributes)
    }

    async fn get_permission(&self, path: &PathBuf) -> Result<Permissions, Error>;

    async fn set_permission(&self, path: &PathBuf, permissions: Permissions) -> Result<(), Error>;

    async fn copy_permission(&self, source: &PathBuf, destination: &PathBuf) -> Result<(), Error> {
        let source_permissions = self.get_permission(source).await?;
        self.set_permission(destination, source_permissions).await?;
        Ok(())
    }

    async fn acquire_file_lock(&self, path: &PathBuf) -> Result<FileLock, Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|err| IOError::SemaphoreClosed(err))?;

        let file_lock = FileLock::new(path).await?;

        Ok(file_lock)
    }

    async fn calculate_hash(&self, path: &PathBuf, hash_type: HashType) -> Result<Vec<u8>, Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|err| IOError::SemaphoreClosed(err))?;

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
        .map_err(|err| SystemError::ThreadPanic(err))??;
        Ok(hash)
    }

    async fn standard_compare(
        &self,
        source: &PathBuf,
        destination: &PathBuf,
    ) -> Result<bool, Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|err| IOError::SemaphoreClosed(err))?;

        let source_metadata =
            fs::metadata(source)
                .await
                .map_err(|err| IOError::GetMetadataFailed(source.clone(), err))?;
        let destination_metadata =
            fs::metadata(destination)
                .await
                .map_err(|err| IOError::GetMetadataFailed(destination.clone(), err))?;

        if source_metadata.len() != destination_metadata.len() {
            return Ok(false);
        }
        let source_modified =
            source_metadata
                .modified()
                .map_err(|err| IOError::GetMetadataFailed(source.clone(), err))?;
        let destination_modified =
            destination_metadata
                .modified()
                .map_err(|err| IOError::GetMetadataFailed(destination.clone(), err))?;
        if source_modified != destination_modified {
            return Ok(false);
        }
        Ok(true)
    }

    async fn advance_compare(
        &self,
        source: &PathBuf,
        destination: &PathBuf,
    ) -> Result<bool, Error> {
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
    ) -> Result<bool, Error> {
        if !self.advance_compare(source, destination).await? {
            return Ok(false);
        }
        let source_file_hash = self.calculate_hash(source, hash_type).await?;
        let destination_file_hash = self.calculate_hash(destination, hash_type).await?;

        Ok(source_file_hash == destination_file_hash)
    }
}

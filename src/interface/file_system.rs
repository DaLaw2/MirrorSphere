use crate::model::error::io::IOError;
use crate::model::error::system::SystemError;
use crate::model::error::Error;
use crate::model::backup::backup_execution::HashType;
use crate::platform::attributes::*;
use crate::utils::file_hash::*;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
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

    async fn is_symlink(&self, path: &Path) -> Result<bool, Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(IOError::SemaphoreClosed)?;

        let symlink_metadata = tokio::fs::symlink_metadata(path)
            .await
            .map_err(|err| IOError::GetMetadataFailed(path, err))?;

        Ok(symlink_metadata.file_type().is_symlink())
    }

    async fn copy_symlink(
        &self,
        source_link: &Path,
        destination_link: &Path,
    ) -> Result<(), Error>;

    async fn list_directory(&self, path: &Path) -> Result<Vec<PathBuf>, Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(IOError::SemaphoreClosed)?;

        let mut result = Vec::new();
        let reader = fs::read_dir(path)
            .await
            .map_err(|err| IOError::ReadDirectoryFailed(path, err))?;
        let mut entries = ReadDirStream::new(reader);
        while let Some(entry) = entries.next().await {
            let path = entry
                .map_err(|err| IOError::ReadDirectoryFailed(path, err))?
                .path();
            result.push(path);
        }
        Ok(result)
    }

    async fn create_directory(&self, path: &Path) -> Result<(), Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(IOError::SemaphoreClosed)?;

        fs::create_dir_all(path)
            .await
            .map_err(|err| IOError::CreateDirectoryFailed(path, err))?;
        Ok(())
    }

    async fn delete_directory(&self, path: &Path) -> Result<(), Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(IOError::SemaphoreClosed)?;

        fs::remove_dir_all(path)
            .await
            .map_err(|err| IOError::DeleteDirectoryFailed(path, err))?;
        Ok(())
    }

    async fn copy_file(&self, source: &Path, destination: &Path) -> Result<(), Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(IOError::SemaphoreClosed)?;

        fs::copy(source, destination)
            .await
            .map_err(|err| IOError::CopyFileFailed(source, destination, err))?;
        Ok(())
    }

    async fn delete_file(&self, path: &Path) -> Result<(), Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(IOError::SemaphoreClosed)?;

        fs::remove_file(path)
            .await
            .map_err(|err| IOError::DeleteFileFailed(path, err))?;
        Ok(())
    }

    async fn get_attributes(&self, path: &Path) -> Result<Attributes, Error>;

    async fn set_attributes(&self, path: &Path, attributes: Attributes) -> Result<(), Error>;

    async fn copy_attributes(&self, source: &Path, destination: &Path) -> Result<(), Error> {
        let source_attributes = self.get_attributes(source).await?;
        self.set_attributes(destination, source_attributes).await?;
        Ok(())
    }

    async fn compare_attributes(
        &self,
        source: &Path,
        destination: &Path,
    ) -> Result<bool, Error> {
        let source_attributes = self.get_attributes(source).await?;
        let destination_attributes = self.get_attributes(destination).await?;
        Ok(source_attributes == destination_attributes)
    }

    async fn get_permission(&self, path: &Path) -> Result<Permissions, Error>;

    async fn set_permission(&self, path: &Path, permissions: Permissions) -> Result<(), Error>;

    async fn copy_permission(&self, source: &Path, destination: &Path) -> Result<(), Error> {
        let source_permissions = self.get_permission(source).await?;
        self.set_permission(destination, source_permissions).await?;
        Ok(())
    }

    async fn calculate_hash(&self, path: &Path, hash_type: HashType) -> Result<Vec<u8>, Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(IOError::SemaphoreClosed)?;

        let path = path.to_path_buf();
        let hash = spawn_blocking(move || {
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
        .map_err(SystemError::ThreadPanic)??;
        Ok(hash)
    }

    async fn standard_compare(
        &self,
        source: &Path,
        destination: &Path,
    ) -> Result<bool, Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(IOError::SemaphoreClosed)?;

        let source_metadata =
            fs::metadata(source)
                .await
                .map_err(|err| IOError::GetMetadataFailed(source, err))?;
        let destination_metadata =
            fs::metadata(destination)
                .await
                .map_err(|err| IOError::GetMetadataFailed(destination, err))?;

        if source_metadata.len() != destination_metadata.len() {
            return Ok(false);
        }
        let source_modified =
            source_metadata
                .modified()
                .map_err(|err| IOError::GetMetadataFailed(source, err))?;
        let destination_modified =
            destination_metadata
                .modified()
                .map_err(|err| IOError::GetMetadataFailed(destination, err))?;
        if source_modified != destination_modified {
            return Ok(false);
        }
        Ok(true)
    }

    async fn advance_compare(
        &self,
        source: &Path,
        destination: &Path,
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
        source: &Path,
        destination: &Path,
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

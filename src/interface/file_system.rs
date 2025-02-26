use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use async_trait::async_trait;
use tokio::sync::Semaphore;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReadDirStream;

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

    async fn create_directory(&self, path: PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore.acquire_owned().await?;
        fs::create_dir_all(path).await?;
        Ok(())
    }

    async fn copy_file(&self, source: PathBuf, destination: PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore.acquire_owned().await?;
        fs::copy(source, destination).await?;
        Ok(())
    }

    async fn delete_directory(&self, path: PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore.acquire_owned().await?;
        fs::remove_dir_all(path).await?;
        Ok(())
    }

    async fn delete_file(&self, path: PathBuf) -> anyhow::Result<()> {
        let semaphore = self.semaphore();
        let _permit = semaphore.acquire_owned().await?;
        fs::remove_file(path).await?;
        Ok(())
    }
}

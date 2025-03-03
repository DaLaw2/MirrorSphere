use crate::core::event_system::event_bus::EventBus;
use crate::interface::file_system::FileSystemTrait;
use crate::model::event::io::attributes::{GetAttributesEvent, SetAttributesEvent};
use crate::model::event::io::permission::GetPermissionEvent;
use crate::platform::attributes::{AdvancedAttributes, Attributes, PermissionAttributes};
use crate::utils::log_entry::io::IOEntry;
use std::os::windows::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;

#[cfg(target_os = "windows")]
pub struct FileSystem {
    semaphore: Arc<Semaphore>,
}

#[cfg(target_os = "windows")]
impl FileSystemTrait for FileSystem {
    fn new(semaphore: Arc<Semaphore>) -> Self {
        FileSystem { semaphore }
    }

    fn semaphore(&self) -> Arc<Semaphore> {
        self.semaphore.clone()
    }

    async fn get_attributes(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<Attributes> {
        let metadata = tokio::fs::metadata(path)
            .await
            .map_err(|_| IOEntry::GetMetadataFailed)?;

        let (read_only, hidden, system, archive) = {
            let attributes = metadata.file_attributes();
            (
                (attributes & 0x1) != 0,
                (attributes & 0x2) != 0,
                (attributes & 0x4) != 0,
                (attributes & 0x32) != 0,
            )
        };

        let creation_time = metadata
            .created()
            .map_err(|_| IOEntry::GetMetadataFailed)?;
        let last_access_time = metadata
            .accessed()
            .map_err(|_| IOEntry::GetMetadataFailed)?;
        let change_time = metadata
            .modified()
            .map_err(|_| IOEntry::GetMetadataFailed)?;

        let attributes = Attributes {
            read_only,
            hidden,
            system,
            archive,
            creation_time,
            last_access_time,
            change_time,
        };

        let event = GetAttributesEvent { task_id, path };
        EventBus::publish(event).await?;

        Ok(attributes)
    }

    async fn get_advanced_attributes(
        &self,
        task_id: Uuid,
        path: PathBuf,
    ) -> anyhow::Result<AdvancedAttributes> {
        let metadata = tokio::fs::metadata(path)
            .await
            .map_err(|_| IOEntry::GetMetadataFailed)?;

        let (read_only, hidden, system, archive, compression, index, encryption) = {
            let attributes = metadata.file_attributes();
            (
                (attributes & 0x1) != 0,
                (attributes & 0x2) != 0,
                (attributes & 0x4) != 0,
                (attributes & 0x32) != 0,
                (attributes & 0x2048) != 0,
                (attributes & 0x8192) != 0,
                (attributes & 0x16384) != 0,
            )
        };

        let creation_time = metadata
            .created()
            .map_err(|_| IOEntry::GetMetadataFailed)?;
        let last_access_time = metadata
            .accessed()
            .map_err(|_| IOEntry::GetMetadataFailed)?;
        let change_time = metadata
            .modified()
            .map_err(|_| IOEntry::GetMetadataFailed)?;

        let attributes = AdvancedAttributes {
            read_only,
            hidden,
            system,
            archive,
            compression,
            index,
            encryption,
            creation_time,
            last_access_time,
            change_time,
        };

        let event = GetAttributesEvent { task_id, path };
        EventBus::publish(event).await?;

        Ok(attributes)
    }

    async fn set_attributes(
        &self,
        task_id: Uuid,
        path: PathBuf,
        attributes: Attributes,
    ) -> anyhow::Result<()> {
        let event = SetAttributesEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(())
    }

    async fn set_advanced_attributes(
        &self,
        task_id: Uuid,
        path: PathBuf,
        attributes: AdvancedAttributes,
    ) -> anyhow::Result<()> {
        let event = SetAttributesEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(())
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
        let event = GetPermissionEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok()
    }

    async fn set_permission(
        &self,
        task_id: Uuid,
        permission: PermissionAttributes,
    ) -> anyhow::Result<()> {
        let event = SetAttributesEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(())
    }
}

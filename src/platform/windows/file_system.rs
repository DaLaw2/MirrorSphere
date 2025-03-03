use std::fs;
use crate::core::event_system::event_bus::EventBus;
use crate::interface::file_system::FileSystemTrait;
use crate::model::event::io::attributes::GetAttributesEvent;
use crate::platform::attributes::{AdvancedAttributes, Attributes};
use crate::utils::log_entry::io::IOEntry;
use std::os::windows::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;
use windows_acl::acl::ACL;

pub struct FileSystem {
    semaphore: Arc<Semaphore>,
}

impl FileSystemTrait for FileSystem {
    fn new(semaphore: Arc<Semaphore>) -> Self {
        FileSystem { semaphore }
    }

    fn semaphore(&self) -> Arc<Semaphore> {
        self.semaphore.clone()
    }

    #[cfg(target_os = "windows")]
    async fn get_attributes(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<Attributes> {
        let metadata = tokio::fs::metadata(path).await
            .map_err(|_| IOEntry::GetMetadataFailed)?;

        let (read_only, hidden, system, archive) = {
            let attributes = metadata.file_attributes();
            (
                (attributes & 0x1) != 0,
                (attributes & 0x2) != 0,
                (attributes & 0x4) != 0,
                (attributes & 0x20) != 0
            )
        };

        let creation_time = metadata.created()
            .map_err(|_| IOEntry::GetMetadataFailed)?;
        let last_access_time = metadata.accessed()
            .map_err(|_| IOEntry::GetMetadataFailed)?;
        let change_time = metadata.modified()
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

        let event = GetAttributesEvent {
            task_id,
            path,
        };
        EventBus::publish(event).await?;

        Ok(attributes)
    }

    #[cfg(target_os = "windows")]
    async fn get_advanced_attributes(
        &self,
        task_id: Uuid,
        path: PathBuf,
    ) -> anyhow::Result<AdvancedAttributes> {
        let metadata = tokio::fs::metadata(path).await
            .map_err(|_| IOEntry::GetMetadataFailed)?;

        let (read_only, hidden, system, archive, compression, encryption, index) = {
            use std::os::windows::fs::MetadataExt;
            let attributes = metadata.file_attributes();
            (
                (attributes & 0x1) != 0,
                (attributes & 0x2) != 0,
                (attributes & 0x4) != 0,
                (attributes & 0x20) != 0,
                (attributes & 0x800) != 0,
                (attributes & 0x4000) != 0,
                (attributes & 0x2000) != 0,
            )
        };

        let creation_time = metadata.created()
            .map_err(|_| IOEntry::GetMetadataFailed)?;
        let last_access_time = metadata.accessed()
            .map_err(|_| IOEntry::GetMetadataFailed)?;
        let change_time = metadata.modified()
            .map_err(|_| IOEntry::GetMetadataFailed)?;

        let (owner, access_control_list) = {
            let path_str = path.to_string_lossy().to_string();
            let acl = ACL::from_file_path(&path_str, true)
                .map_err(|_| IOEntry::GetMetadataFailed)?;
            let owner = String::new();
            (owner, acl)
        };

        let attributes = AdvancedAttributes {
            read_only,
            hidden,
            system,
            archive,
            compression,
            encryption,
            index,
            creation_time,
            last_access_time,
            change_time,
            owner,
            access_control_list,
        };

        let event = GetAttributesEvent {
            task_id,
            path,
        };
        EventBus::publish(event).await?;

        Ok(attributes)
    }

    async fn set_attributes(
        &self,
        task_id: Uuid,
        path: PathBuf,
        attributes: Attributes,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn set_advanced_attributes(
        &self,
        task_id: Uuid,
        path: PathBuf,
        attributes: AdvancedAttributes,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn compare_attributes(
        &self,
        source: PathBuf,
        destination: PathBuf,
    ) -> anyhow::Result<bool> {
        todo!()
    }

    async fn compare_advanced_attributes(
        &self,
        source: PathBuf,
        destination: PathBuf,
    ) -> anyhow::Result<bool> {
        todo!()
    }
}

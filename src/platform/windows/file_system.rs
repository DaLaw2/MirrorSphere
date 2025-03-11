use crate::core::event_system::event_bus::EventBus;
use crate::interface::file_system::FileSystemTrait;
use crate::model::event::io::attributes::{GetAttributesEvent, SetAttributesEvent};
use crate::model::event::io::permission::GetPermissionEvent;
use crate::platform::attributes::{Attributes, Permissions};
use crate::platform::windows::helper::system_time_to_file_time;
use crate::utils::log_entry::io::IOEntry;
use async_trait::async_trait;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;
use windows::core::imp::CloseHandle;
use windows::core::PCWSTR;
use windows::Win32::Foundation::GENERIC_ALL;
use windows::Win32::Storage::FileSystem::{
    CreateFileW, SetFileAttributesW, SetFileTime, FILE_ATTRIBUTE_ARCHIVE, FILE_ATTRIBUTE_HIDDEN,
    FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_NOT_CONTENT_INDEXED, FILE_ATTRIBUTE_READONLY,
    FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};

#[cfg(target_os = "windows")]
pub struct FileSystem {
    semaphore: Arc<Semaphore>,
}

#[async_trait]
#[cfg(target_os = "windows")]
impl FileSystemTrait for FileSystem {
    fn new(semaphore: Arc<Semaphore>) -> Self {
        FileSystem { semaphore }
    }

    fn semaphore(&self) -> Arc<Semaphore> {
        self.semaphore.clone()
    }

    async fn get_attributes(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<Attributes> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOEntry::SemaphoreClosed)?;

        let metadata = tokio::fs::metadata(&path)
            .await
            .map_err(|_| IOEntry::GetMetadataFailed)?;

        let (read_only, hidden, archive, normal, index) = {
            let attributes = metadata.file_attributes();
            (
                (attributes & FILE_ATTRIBUTE_READONLY.0) != 0,
                (attributes & FILE_ATTRIBUTE_HIDDEN.0) != 0,
                (attributes & FILE_ATTRIBUTE_ARCHIVE.0) != 0,
                (attributes & FILE_ATTRIBUTE_NORMAL.0) != 0,
                (attributes & FILE_ATTRIBUTE_NOT_CONTENT_INDEXED.0) != 0,
            )
        };

        let creation_time = metadata.created().map_err(|_| IOEntry::GetMetadataFailed)?;
        let last_access_time = metadata
            .accessed()
            .map_err(|_| IOEntry::GetMetadataFailed)?;
        let change_time = metadata
            .modified()
            .map_err(|_| IOEntry::GetMetadataFailed)?;

        let attributes = Attributes {
            read_only,
            hidden,
            archive,
            normal,
            index,
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
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|_| IOEntry::SemaphoreClosed)?;

        let file_path_wild: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();

        let mut file_attributes: u32 = 0;
        file_attributes |= FILE_ATTRIBUTE_READONLY.0;
        file_attributes |= FILE_ATTRIBUTE_HIDDEN.0;
        file_attributes |= FILE_ATTRIBUTE_ARCHIVE.0;
        file_attributes |= FILE_ATTRIBUTE_NORMAL.0;
        file_attributes |= FILE_ATTRIBUTE_NOT_CONTENT_INDEXED.0;

        unsafe {
            SetFileAttributesW(
                PCWSTR(file_path_wild.as_ptr()),
                FILE_FLAGS_AND_ATTRIBUTES(file_attributes),
            )
            .map_err(|_| IOEntry::SetMetadataFailed)?;

            let handle = CreateFileW(
                PCWSTR(file_path_wild.as_ptr()),
                GENERIC_ALL.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                None,
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(file_attributes),
                None,
            )
            .map_err(|_| IOEntry::SetMetadataFailed)?;

            let creation_filetime = system_time_to_file_time(attributes.creation_time)?;
            let last_access_filetime = system_time_to_file_time(attributes.last_access_time)?;
            let change_filetime = system_time_to_file_time(attributes.change_time)?;

            let result = SetFileTime(
                handle,
                Some(&creation_filetime),
                Some(&last_access_filetime),
                Some(&change_filetime),
            );

            CloseHandle(handle.0);

            result.map_err(|_| IOEntry::SetMetadataFailed)?;
        }

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

    async fn get_permission(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<Permissions> {
        let event = GetPermissionEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok()
    }

    async fn set_permission(
        &self,
        task_id: Uuid,
        path: PathBuf,
        permissions: Permissions,
    ) -> anyhow::Result<()> {
        let event = SetAttributesEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(())
    }
}

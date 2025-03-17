use crate::core::event_system::event_bus::EventBus;
use crate::interface::file_system::FileSystemTrait;
use crate::model::event::io::attributes::{GetAttributesEvent, SetAttributesEvent};
use crate::model::event::io::permission::GetPermissionEvent;
use crate::platform::attributes::{Attributes, Permissions};
use crate::utils::log_entry::io::IOEntry;
use crate::utils::log_entry::system::SystemEntry;
use async_trait::async_trait;
use chrono::{DateTime, Datelike, Timelike};
use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::MetadataExt;
use std::os::windows::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Semaphore;
use tokio::task::spawn_blocking;
use uuid::Uuid;
use windows::core::imp::CloseHandle;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{LocalFree, ERROR_SUCCESS, FILETIME, GENERIC_ALL, HANDLE, HLOCAL, SYSTEMTIME};
use windows::Win32::Security::Authorization::{GetNamedSecurityInfoW, GetSecurityInfo, SetSecurityInfo, SE_FILE_OBJECT, SE_OBJECT_TYPE};
use windows::Win32::Security::{CopySid, GetLengthSid, LookupAccountSidW, ACL, DACL_SECURITY_INFORMATION, OBJECT_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION, PROTECTED_SACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID, SACL_SECURITY_INFORMATION, SID_NAME_USE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, SetFileAttributesW, SetFileTime, FILE_ATTRIBUTE_ARCHIVE, FILE_ATTRIBUTE_HIDDEN,
    FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_NOT_CONTENT_INDEXED, FILE_ATTRIBUTE_READONLY,
    FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Time::SystemTimeToFileTime;

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

        spawn_blocking(move || unsafe {
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

            let creation_filetime = Self::system_time_to_file_time(attributes.creation_time)?;
            let last_access_filetime = Self::system_time_to_file_time(attributes.last_access_time)?;
            let change_filetime = Self::system_time_to_file_time(attributes.change_time)?;

            let result = SetFileTime(
                handle,
                Some(&creation_filetime),
                Some(&last_access_filetime),
                Some(&change_filetime),
            );

            CloseHandle(handle.0);

            result.map_err(|_| IOEntry::SetMetadataFailed)?;
        })
            .await
            .map_err(|_| SystemEntry::ThreadPanic)?;

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
        let source_attributes = self.get_attributes(task_id, source).await?;
        let destination_attributes = self.get_attributes(task_id, destination).await?;
        Ok(source_attributes == destination_attributes)
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

impl FileSystem {
    pub async fn get_advanced_permission(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<Permissions> {
        let event = GetPermissionEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok()
    }

    async fn set_advanced_permission(
        &self,
        task_id: Uuid,
        path: PathBuf,
        permissions: Permissions,
    ) -> anyhow::Result<()> {
        let event = SetAttributesEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(())
    }

    fn get_security_descriptor(
        handle: HANDLE,
        security_info: OBJECT_SECURITY_INFORMATION,
        object_type: SE_OBJECT_TYPE,
    ) -> anyhow::Result<PSECURITY_DESCRIPTOR> {
        let mut security_descriptor = PSECURITY_DESCRIPTOR::default();

        unsafe {
            let status = GetSecurityInfo(
                handle,
                object_type,
                security_info,
                None,
                None,
                None,
                None,
                Some(&mut security_descriptor),
            );

            if status.is_err() {
                Err(IOEntry::GetMetadataFailed)?;
            }

            Ok(security_descriptor)
        }
    }

    fn convert_inherited_to_explicit(
        handle: HANDLE,
        object_type: SE_OBJECT_TYPE,
        security_info: OBJECT_SECURITY_INFORMATION,
    ) -> anyhow::Result<()> {
        unsafe {
            let security_descriptor = Self::get_security_descriptor(handle, security_info, object_type)?;

            let protect_flags = if security_info.contains(DACL_SECURITY_INFORMATION) {
                PROTECTED_DACL_SECURITY_INFORMATION
            } else if security_info.contains(SACL_SECURITY_INFORMATION) {
                PROTECTED_SACL_SECURITY_INFORMATION
            } else {
                Err(SystemEntry::UnknownError)?
            };

            let status = SetSecurityInfo(
                handle,
                object_type,
                security_info | protect_flags,
                None,
                None,
                None,
                None,
            );

            if status.is_err() {
                return Err(IOEntry::GetMetadataFailed)?;
            }

            Ok(())
        }
    }

    fn system_time_to_file_time(system_time: SystemTime) -> anyhow::Result<FILETIME> {
        let duration = system_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| SystemEntry::InternalError)?;

        let epoch = DateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
            .ok_or_else(|| SystemEntry::InternalError)?;

        let sys_time = SYSTEMTIME {
            wYear: epoch.year() as u16,
            wMonth: epoch.month() as u16,
            wDayOfWeek: epoch.weekday().num_days_from_sunday() as u16,
            wDay: epoch.day() as u16,
            wHour: epoch.hour() as u16,
            wMinute: epoch.minute() as u16,
            wSecond: epoch.second() as u16,
            wMilliseconds: (duration.subsec_nanos() / 1_000_000) as u16,
        };

        let mut file_time = FILETIME::default();

        unsafe {
            SystemTimeToFileTime(&sys_time, &mut file_time)
                .map_err(|_| SystemEntry::InternalError)?;
            Ok(file_time)
        }
    }
}

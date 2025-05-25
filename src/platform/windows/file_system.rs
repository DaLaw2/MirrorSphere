use crate::core::event_system::event_bus::EventBus;
use crate::interface::file_system::FileSystemTrait;
use crate::model::error::io::IOError;
use crate::model::error::misc::MiscError;
use crate::model::error::system::SystemError;
use crate::model::event::io::attributes::{GetAttributesEvent, SetAttributesEvent};
use crate::model::event::io::permission::GetPermissionEvent;
use crate::platform::attributes::{Attributes, Permissions};
use crate::platform::raii_guard::SecurityDescriptorGuard;
use async_trait::async_trait;
use chrono::{DateTime, Datelike, Timelike};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::MetadataExt;
use std::path::PathBuf;
use std::ptr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Semaphore;
use tokio::task::spawn_blocking;
use uuid::Uuid;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, FILETIME, GENERIC_ALL, SYSTEMTIME};
use windows::Win32::Security::Authorization::{
    GetNamedSecurityInfoW, SetNamedSecurityInfoW, SE_FILE_OBJECT,
};
use windows::Win32::Security::{ACL, BACKUP_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, SetFileAttributesW, SetFileTime, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_DELETE,
    FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Time::SystemTimeToFileTime;

pub struct FileSystem {
    semaphore: Arc<Semaphore>,
}

#[async_trait]
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
            .map_err(|_| IOError::SemaphoreClosed)?;

        let metadata = tokio::fs::metadata(&path)
            .await
            .map_err(|_| IOError::GetMetadataFailed)?;

        let attributes = metadata.file_attributes();

        let creation_time = metadata
            .created()
            .map_err(|_| IOError::GetMetadataFailed)?;
        let last_access_time = metadata
            .accessed()
            .map_err(|_| IOError::GetMetadataFailed)?;
        let change_time = metadata
            .modified()
            .map_err(|_| IOError::GetMetadataFailed)?;

        let attributes = Attributes {
            attributes,
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
            .map_err(|_| IOError::SemaphoreClosed)?;

        let file_path_wild: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();

        let file_attributes = attributes.attributes;

        spawn_blocking(move || unsafe {
            SetFileAttributesW(
                PCWSTR(file_path_wild.as_ptr()),
                FILE_FLAGS_AND_ATTRIBUTES(file_attributes),
            )
            .map_err(|_| IOError::SetMetadataFailed)?;

            let handle = CreateFileW(
                PCWSTR(file_path_wild.as_ptr()),
                GENERIC_ALL.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                None,
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(file_attributes),
                None,
            )
            .map_err(|_| IOError::SetMetadataFailed)?;

            let creation_filetime = Self::system_time_to_file_time(attributes.creation_time)?;
            let last_access_filetime = Self::system_time_to_file_time(attributes.last_access_time)?;
            let change_filetime = Self::system_time_to_file_time(attributes.change_time)?;

            let result = SetFileTime(
                handle,
                Some(&creation_filetime),
                Some(&last_access_filetime),
                Some(&change_filetime),
            );

            CloseHandle(handle).map_err(|_| MiscError::ObjectFreeFailed)?;

            result.map_err(|_| IOError::SetMetadataFailed)?;

            Ok::<(), anyhow::Error>(())
        })
        .await
        .map_err(|_| SystemError::ThreadPanic)??;

        let event = SetAttributesEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(())
    }

    async fn get_permission(&self, task_id: Uuid, path: PathBuf) -> anyhow::Result<Permissions> {
        let file_path_wild: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();

        let permission = spawn_blocking(move || unsafe {
            let security_info = BACKUP_SECURITY_INFORMATION;
            let mut owner = PSID::default();
            let mut primary_group = PSID::default();
            let mut dacl: *mut ACL = ptr::null_mut();
            let mut sacl: *mut ACL = ptr::null_mut();
            let mut security_descriptor = PSECURITY_DESCRIPTOR::default();

            if GetNamedSecurityInfoW(
                PCWSTR(file_path_wild.as_ptr()),
                SE_FILE_OBJECT,
                security_info,
                Some(&mut owner),
                Some(&mut primary_group),
                Some(&mut dacl),
                Some(&mut sacl),
                &mut security_descriptor,
            )
            .is_err()
            {
                Err(IOError::GetMetadataFailed)?;
            }

            Ok::<Permissions, anyhow::Error>(Permissions {
                owner,
                primary_group,
                dacl,
                sacl,
                security_descriptor: SecurityDescriptorGuard::new(security_descriptor),
            })
        })
        .await
        .map_err(|_| SystemError::ThreadPanic)??;

        let event = GetPermissionEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(permission)
    }

    async fn set_permission(
        &self,
        task_id: Uuid,
        path: PathBuf,
        permissions: Permissions,
    ) -> anyhow::Result<()> {
        let file_path_wild: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();

        let security_info = BACKUP_SECURITY_INFORMATION;
        let owner = permissions.owner;
        let primary_group = permissions.primary_group;
        let dacl = permissions.dacl;
        let sacl = permissions.sacl;
        let security_descriptor = permissions.security_descriptor;

        unsafe {
            if SetNamedSecurityInfoW(
                PCWSTR(file_path_wild.as_ptr()),
                SE_FILE_OBJECT,
                security_info,
                Some(owner),
                Some(primary_group),
                Some(dacl),
                Some(sacl),
            )
            .is_err()
            {
                Err(IOError::SetMetadataFailed)?;
            }
        }

        drop(security_descriptor);

        let event = SetAttributesEvent { task_id, path };
        EventBus::publish(event).await?;
        Ok(())
    }
}

impl FileSystem {
    fn system_time_to_file_time(system_time: SystemTime) -> anyhow::Result<FILETIME> {
        let duration = system_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| SystemError::InternalError)?;

        let epoch = DateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
            .ok_or_else(|| SystemError::InternalError)?;

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
                .map_err(|_| SystemError::InternalError)?;
            Ok(file_time)
        }
    }
}

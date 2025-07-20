use crate::interface::file_system::FileSystemTrait;
use crate::model::error::io::IOError;
use crate::model::error::misc::MiscError;
use crate::model::error::system::SystemError;
use crate::model::error::Error;
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
        Self { semaphore }
    }

    fn semaphore(&self) -> Arc<Semaphore> {
        self.semaphore.clone()
    }

    async fn create_symlink(&self, target: &PathBuf, link_path: &PathBuf) -> Result<(), Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|err| IOError::SemaphoreClosed(err))?;

        if link_path.is_dir() {
            tokio::fs::symlink_dir(target, link_path).await
        } else {
            tokio::fs::symlink_file(target, link_path).await
        }
        .map_err(|err| IOError::CreateSymbolLinkFailed(target.clone(), link_path.clone(), err))?;

        Ok(())
    }

    async fn copy_symlink(
        &self,
        source_link: &PathBuf,
        destination_link: &PathBuf,
    ) -> Result<(), Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|err| IOError::SemaphoreClosed(err))?;

        let link_target =
            tokio::fs::read_link(source_link)
                .await
                .map_err(|err| IOError::ReadSymbolLinkFailed(source_link.clone(), err))?;

        if link_target.is_dir() {
            tokio::fs::symlink_dir(&link_target, destination_link).await
        } else {
            tokio::fs::symlink_file(&link_target, destination_link).await
        }
        .map_err(|err| IOError::CreateSymbolLinkFailed(link_target, destination_link, err))?;

        Ok(())
    }

    async fn get_attributes(&self, path: &PathBuf) -> Result<Attributes, Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|err| IOError::SemaphoreClosed(err))?;

        let metadata = tokio::fs::metadata(path)
            .await
            .map_err(|err| IOError::GetMetadataFailed(path.clone(), err))?;

        let attributes = metadata.file_attributes();

        let creation_time = metadata
            .created()
            .map_err(|err| IOError::GetMetadataFailed(path.clone(), err))?;
        let last_access_time = metadata
            .accessed()
            .map_err(|err| IOError::GetMetadataFailed(path.clone(), err))?;
        let change_time = metadata
            .modified()
            .map_err(|err| IOError::GetMetadataFailed(path.clone(), err))?;

        let attributes = Attributes {
            attributes,
            creation_time,
            last_access_time,
            change_time,
        };

        Ok(attributes)
    }

    async fn set_attributes(&self, path: &PathBuf, attributes: Attributes) -> Result<(), Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(|err| IOError::SemaphoreClosed(err))?;

        let file_path_wild: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();

        let file_attributes = attributes.attributes;

        let path = path.clone();
        spawn_blocking(move || unsafe {
            SetFileAttributesW(
                PCWSTR(file_path_wild.as_ptr()),
                FILE_FLAGS_AND_ATTRIBUTES(file_attributes),
            )
            .map_err(|err| IOError::SetMetadataFailed(path.clone(), err))?;

            let handle = CreateFileW(
                PCWSTR(file_path_wild.as_ptr()),
                GENERIC_ALL.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                None,
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(file_attributes),
                None,
            )
            .map_err(|err| IOError::SetMetadataFailed(path.clone(), err))?;

            let creation_filetime = Self::system_time_to_file_time(attributes.creation_time)?;
            let last_access_filetime = Self::system_time_to_file_time(attributes.last_access_time)?;
            let change_filetime = Self::system_time_to_file_time(attributes.change_time)?;

            let result = SetFileTime(
                handle,
                Some(&creation_filetime),
                Some(&last_access_filetime),
                Some(&change_filetime),
            );

            CloseHandle(handle).map_err(|err| MiscError::ObjectFreeFailed(err))?;

            result.map_err(|err| IOError::SetMetadataFailed(path.clone(), err))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|err| SystemError::ThreadPanic(err))??;

        Ok(())
    }

    async fn get_permission(&self, path: &PathBuf) -> Result<Permissions, Error> {
        let file_path_wild: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();

        let path = path.clone();
        let permission = spawn_blocking(move || unsafe {
            let security_info = BACKUP_SECURITY_INFORMATION;
            let mut owner = PSID::default();
            let mut primary_group = PSID::default();
            let mut dacl: *mut ACL = ptr::null_mut();
            let mut sacl: *mut ACL = ptr::null_mut();
            let mut security_descriptor = PSECURITY_DESCRIPTOR::default();

            let result = GetNamedSecurityInfoW(
                PCWSTR(file_path_wild.as_ptr()),
                SE_FILE_OBJECT,
                security_info,
                Some(&mut owner),
                Some(&mut primary_group),
                Some(&mut dacl),
                Some(&mut sacl),
                &mut security_descriptor,
            );

            if result.is_err() {
                Err(IOError::GetMetadataFailed(path.clone(), format!("{:?}", result)))?;
            }

            Ok::<Permissions, Error>(Permissions {
                owner,
                primary_group,
                dacl,
                sacl,
                security_descriptor: SecurityDescriptorGuard::new(security_descriptor),
            })
        })
        .await
        .map_err(|err| SystemError::ThreadPanic(err))??;

        Ok(permission)
    }

    async fn set_permission(&self, path: &PathBuf, permissions: Permissions) -> Result<(), Error> {
        let file_path_wild: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();

        let security_info = BACKUP_SECURITY_INFORMATION;
        let owner = permissions.owner;
        let primary_group = permissions.primary_group;
        let dacl = permissions.dacl;
        let sacl = permissions.sacl;
        let security_descriptor = permissions.security_descriptor;

        unsafe {
            let result = SetNamedSecurityInfoW(
                PCWSTR(file_path_wild.as_ptr()),
                SE_FILE_OBJECT,
                security_info,
                Some(owner),
                Some(primary_group),
                Some(dacl),
                Some(sacl),
            );

            if result.is_err() {
                Err(IOError::SetMetadataFailed(path.clone(), format!("{:?}", result)))?;
            }
        }

        drop(security_descriptor);

        Ok(())
    }
}

impl FileSystem {
    fn system_time_to_file_time(system_time: SystemTime) -> Result<FILETIME, Error> {
        let duration = system_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|err| SystemError::UnexpectError(err))?;

        let epoch = DateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
            .ok_or(SystemError::UnknownError)?;

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
                .map_err(|err| SystemError::UnexpectError(err))?;
            Ok(file_time)
        }
    }
}

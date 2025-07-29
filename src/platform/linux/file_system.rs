use crate::interface::file_system::FileSystemTrait;
use crate::model::error::Error;
use crate::model::error::io::IOError;
use crate::model::error::system::SystemError;
use crate::platform::attributes::{Attributes, Permissions};
use async_trait::async_trait;
use libc::mode_t;
use std::ffi::CString;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Semaphore;
use tokio::task::spawn_blocking;

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

    async fn copy_symlink(
        &self,
        source_link: &PathBuf,
        destination_link: &PathBuf,
    ) -> Result<(), Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(IOError::SemaphoreClosed)?;

        tokio::fs::symlink(&source_link, destination_link)
            .await
            .map_err(|err| {
                IOError::CreateSymbolLinkFailed(source_link.clone(), destination_link.clone(), err)
            })?;

        Ok(())
    }

    async fn get_attributes(&self, path: &PathBuf) -> Result<Attributes, Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(IOError::SemaphoreClosed)?;

        let metadata = tokio::fs::metadata(path)
            .await
            .map_err(|err| IOError::GetMetadataFailed(path.clone(), err))?;

        let mode = metadata.mode();
        let file_type = metadata.file_type();

        let mut attributes = 0_u32;

        if file_type.is_dir() {
            attributes |= libc::S_IFDIR;
        } else if file_type.is_file() {
            attributes |= libc::S_IFREG;
        } else if file_type.is_symlink() {
            attributes |= libc::S_IFLNK;
        }

        attributes |= mode & 0o777;

        let creation_time = metadata
            .created()
            .unwrap_or_else(|_| metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH));
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
            .map_err(IOError::SemaphoreClosed)?;

        let path_clone = path.clone();
        let mode = attributes.attributes & 0o7777;

        spawn_blocking(move || {
            let path = path_clone;
            let c_path = CString::new(path.to_string_lossy().as_bytes())
                .map_err(|err| IOError::SetMetadataFailed(path.clone(), err))?;

            unsafe {
                if libc::chmod(c_path.as_ptr(), mode as mode_t) != 0 {
                    Err(IOError::SetMetadataFailed(
                        path.clone(),
                        "libc chmod failed",
                    ))?;
                }
            }

            Self::set_file_times(&path, &attributes)?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(SystemError::ThreadPanic)??;

        Ok(())
    }

    async fn get_permission(&self, path: &PathBuf) -> Result<Permissions, Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(IOError::SemaphoreClosed)?;

        let path_clone = path.clone();

        let permission = spawn_blocking(move || {
            let path = path_clone;
            let metadata = std::fs::metadata(&path)
                .map_err(|err| IOError::GetMetadataFailed(path.clone(), err))?;

            let uid = metadata.uid();
            let gid = metadata.gid();
            let mode = metadata.mode();

            Ok::<Permissions, Error>(Permissions {
                uid,
                gid,
                mode,
                is_sticky: (mode & libc::S_ISVTX as u32) != 0,
                is_setuid: (mode & libc::S_ISUID as u32) != 0,
                is_setgid: (mode & libc::S_ISGID as u32) != 0,
            })
        })
        .await
        .map_err(SystemError::ThreadPanic)??;

        Ok(permission)
    }

    async fn set_permission(&self, path: &PathBuf, permissions: Permissions) -> Result<(), Error> {
        let semaphore = self.semaphore();
        let _permit = semaphore
            .acquire_owned()
            .await
            .map_err(IOError::SemaphoreClosed)?;

        let path_clone = path.clone();

        spawn_blocking(move || {
            let path = path_clone;
            let c_path: CString = CString::new(path.to_string_lossy().as_bytes())
                .map_err(|err| Error::IO(IOError::SetMetadataFailed(path.clone(), err)))?;

            unsafe {
                if libc::chown(c_path.as_ptr(), permissions.uid, permissions.gid) != 0 {
                    return Err(
                        IOError::SetMetadataFailed(path.clone(), "libc chmod failed").into(),
                    );
                }

                let mut mode = permissions.mode & 0o7777;
                if permissions.is_sticky {
                    mode |= libc::S_ISVTX as u32;
                }
                if permissions.is_setuid {
                    mode |= libc::S_ISUID as u32;
                }
                if permissions.is_setgid {
                    mode |= libc::S_ISGID as u32;
                }

                if libc::chmod(c_path.as_ptr(), mode as mode_t) != 0 {
                    return Err(
                        IOError::SetMetadataFailed(path.clone(), "libc chmod failed").into(),
                    );
                }
            }

            Ok::<(), Error>(())
        })
        .await
        .map_err(SystemError::ThreadPanic)??;

        Ok(())
    }
}

impl FileSystem {
    fn set_file_times(path: &PathBuf, attributes: &Attributes) -> Result<(), Error> {
        let c_path = CString::new(path.to_string_lossy().as_bytes())
            .map_err(|err| IOError::SetMetadataFailed(path.clone(), err))?;

        let access_time = Self::system_time_to_timespec(attributes.last_access_time)?;
        let modify_time = Self::system_time_to_timespec(attributes.change_time)?;

        let times = [access_time, modify_time];

        unsafe {
            if libc::utimensat(libc::AT_FDCWD, c_path.as_ptr(), times.as_ptr(), 0) != 0 {
                Err(IOError::SetMetadataFailed(
                    path.clone(),
                    "libc utimensat failed",
                ))?;
            }
        }

        Ok(())
    }

    fn system_time_to_timespec(system_time: SystemTime) -> Result<libc::timespec, Error> {
        let duration = system_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(SystemError::UnexpectError)?;

        Ok(libc::timespec {
            tv_sec: duration.as_secs() as libc::time_t,
            tv_nsec: duration.subsec_nanos() as libc::c_long,
        })
    }
}

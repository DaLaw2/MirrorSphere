use crate::core::io_manager::IOManager;
use crate::interface::file_system::FileSystemTrait;
use crate::model::error::database::DatabaseError;
use crate::model::error::event::EventError;
use crate::model::error::io::IOError;
use crate::model::error::misc::MiscError;
use crate::model::error::serializable::SerializableError;
use crate::model::error::system::SystemError;
use crate::model::error::task::TaskError;
use crate::platform::constants::PROGRESS_SAVE_PATH;
use memmap2::MmapMut;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
struct ProgressData {
    current_level: Vec<PathBuf>,
    errors: Vec<SerializableError>,
}

pub struct ProgressTracker;

impl ProgressTracker {
    pub async fn save_task(
        task_uuid: Uuid,
        current_level: Vec<PathBuf>,
        errors: Vec<anyhow::Error>,
    ) -> anyhow::Result<()> {
        let serializable_errors = Self::convert_errors(errors);

        let progress_data = ProgressData {
            current_level,
            errors: serializable_errors,
        };

        Self::write_progress_file(task_uuid, &progress_data).await
    }

    pub async fn resume_task(task_uuid: Uuid) -> (Vec<PathBuf>, Vec<anyhow::Error>) {
        match Self::read_progress_file(task_uuid).await {
            Ok(progress_data) => {
                let anyhow_errors = Self::convert_back_errors(progress_data.errors);
                (progress_data.current_level, anyhow_errors)
            }
            Err(_) => (Vec::new(), Vec::new()),
        }
    }

    fn convert_errors(errors: Vec<anyhow::Error>) -> Vec<SerializableError> {
        errors
            .into_iter()
            .filter_map(|err| {
                if err.downcast_ref::<TaskError>().is_some() {
                    Some(SerializableError::Task(
                        err.downcast::<TaskError>().unwrap(),
                    ))
                } else if err.downcast_ref::<SystemError>().is_some() {
                    Some(SerializableError::System(
                        err.downcast::<SystemError>().unwrap(),
                    ))
                } else if err.downcast_ref::<IOError>().is_some() {
                    Some(SerializableError::IO(err.downcast::<IOError>().unwrap()))
                } else if err.downcast_ref::<DatabaseError>().is_some() {
                    Some(SerializableError::Database(
                        err.downcast::<DatabaseError>().unwrap(),
                    ))
                } else if err.downcast_ref::<EventError>().is_some() {
                    Some(SerializableError::Event(
                        err.downcast::<EventError>().unwrap(),
                    ))
                } else if err.downcast_ref::<MiscError>().is_some() {
                    Some(SerializableError::Misc(
                        err.downcast::<MiscError>().unwrap(),
                    ))
                } else {
                    MiscError::UnexpectedErrorType.log();
                    unreachable!("All errors should be of known types")
                }
            })
            .collect()
    }

    fn convert_back_errors(errors: Vec<SerializableError>) -> Vec<anyhow::Error> {
        errors
            .into_iter()
            .map(|err| match err {
                SerializableError::Task(err) => anyhow::Error::from(err),
                SerializableError::System(err) => anyhow::Error::from(err),
                SerializableError::IO(err) => anyhow::Error::from(err),
                SerializableError::Database(err) => anyhow::Error::from(err),
                SerializableError::Event(err) => anyhow::Error::from(err),
                SerializableError::Misc(err) => anyhow::Error::from(err),
            })
            .collect()
    }

    async fn write_progress_file(task_uuid: Uuid, data: &ProgressData) -> anyhow::Result<()> {
        let saved_path = PathBuf::from(PROGRESS_SAVE_PATH).join(task_uuid.to_string());

        if let Some(parent) = saved_path.parent() {
            let instance = IOManager::instance();
            let parent = parent.to_path_buf();
            instance.create_directory(&parent).await?;
        }

        let config = bincode::config::standard();
        let serialized = bincode::serde::encode_to_vec(data, config)?;
        let data_len = serialized.len();

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&saved_path)
            .await
            .map_err(|_| IOError::CreateFileFailed {
                path: saved_path.clone(),
            })?;

        file.set_len(data_len as u64)
            .await
            .map_err(|_| IOError::WriteFileFailed {
                path: saved_path.clone(),
            })?;

        let mut mmap = unsafe { MmapMut::map_mut(&file)? };
        mmap[..data_len].copy_from_slice(&serialized);
        mmap.flush()?;

        Ok(())
    }

    async fn read_progress_file(task_uuid: Uuid) -> anyhow::Result<ProgressData> {
        let saved_path = PathBuf::from(PROGRESS_SAVE_PATH).join(task_uuid.to_string());

        if !saved_path.exists() {
            Err(IOError::FileDoesNotExist {
                path: saved_path.clone(),
            })?
        }

        let file =
            tokio::fs::File::open(&saved_path)
                .await
                .map_err(|_| IOError::ReadFileFailed {
                    path: saved_path.clone(),
                })?;

        let mmap = unsafe { MmapMut::map_mut(&file)? };

        let config = bincode::config::standard();
        let (progress_data, _) = bincode::serde::decode_from_slice(&mmap, config)
            .map_err(|_| MiscError::BincodeDecodeError)?;

        Ok(progress_data)
    }
}

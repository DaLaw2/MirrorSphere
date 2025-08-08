use crate::core::infrastructure::io_manager::IOManager;
use crate::interface::file_system::FileSystemTrait;
use crate::model::backup::progress_data::ProgressData;
use crate::model::error::Error;
use crate::model::error::io::IOError;
use crate::model::error::misc::MiscError;
use crate::platform::constants::PROGRESS_SAVE_PATH;
use memmap2::MmapMut;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use uuid::Uuid;

pub struct ProgressTracker {
    io_manager: Arc<IOManager>,
}

impl ProgressTracker {
    pub fn new(io_manager: Arc<IOManager>) -> Self {
        Self { io_manager }
    }

    pub async fn save_execution(
        &self,
        execution_uuid: Uuid,
        current_level: Vec<PathBuf>,
        errors: Vec<Error>,
    ) -> Result<(), Error> {
        let progress_data = ProgressData::new(current_level, errors);

        self.write_progress_file(execution_uuid, &progress_data)
            .await
    }

    pub async fn resume_execution(&self, execution_uuid: Uuid) -> (Vec<PathBuf>, Vec<Error>) {
        match self.read_progress_file(execution_uuid).await {
            Ok(progress_data) => (progress_data.current_level, progress_data.errors),
            Err(_) => (Vec::new(), Vec::new()),
        }
    }

    async fn write_progress_file(
        &self,
        execution_uuid: Uuid,
        data: &ProgressData,
    ) -> Result<(), Error> {
        let saved_path = PathBuf::from(PROGRESS_SAVE_PATH).join(execution_uuid.to_string());

        if let Some(parent) = saved_path.parent() {
            let instance = &self.io_manager;
            let parent = parent.to_path_buf();
            instance.create_directory(&parent).await?;
        }

        let config = bincode::config::standard();
        let serialized = bincode::serde::encode_to_vec(data, config)
            .map_err(MiscError::DeserializeError)?;
        let data_len = serialized.len();

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&saved_path)
            .await
            .map_err(|err| IOError::CreateFileFailed(saved_path.clone(), err))?;

        file.set_len(data_len as u64)
            .await
            .map_err(|err| IOError::WriteFileFailed(saved_path.clone(), err))?;

        let mut mmap = unsafe {
            MmapMut::map_mut(&file)
                .map_err(|err| IOError::WriteFileFailed(saved_path.clone(), err))?
        };
        mmap[..data_len].copy_from_slice(&serialized);
        mmap.flush()
            .map_err(|err| IOError::WriteFileFailed(saved_path, err))?;

        Ok(())
    }

    async fn read_progress_file(&self, execution_uuid: Uuid) -> Result<ProgressData, Error> {
        let saved_path = PathBuf::from(PROGRESS_SAVE_PATH).join(execution_uuid.to_string());

        if !saved_path.exists() {
            Err(IOError::FileDoesNotExist {
                path: saved_path.clone(),
            })?
        }

        let file = tokio::fs::File::open(&saved_path)
            .await
            .map_err(|err| IOError::ReadFileFailed(saved_path.clone(), err))?;

        let mmap = unsafe {
            MmapMut::map_mut(&file).map_err(|err| IOError::ReadFileFailed(saved_path, err))?
        };

        let config = bincode::config::standard();
        let (progress_data, _) = bincode::serde::decode_from_slice(&mmap, config)
            .map_err(MiscError::DeserializeError)?;

        Ok(progress_data)
    }
}

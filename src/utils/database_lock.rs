use crate::model::error::database::DatabaseError;
use crate::model::error::Error;
use crate::platform::constants::DATABASE_LOCK_PATH;
use std::fs;
use tokio::fs::File;

pub struct DatabaseLock {
    _private: (),
}

impl DatabaseLock {
    pub async fn acquire() -> Result<Self, Error> {
        let lock = Self { _private: () };
        if tokio::fs::metadata(DATABASE_LOCK_PATH).await.is_err() {
            File::create(&DATABASE_LOCK_PATH)
                .await
                .map_err(|err| DatabaseError::LockDatabaseFailed(err))?;
            Ok(lock)
        } else {
            Err(DatabaseError::LockDatabaseFailed("Lock file already exists."))?
        }
    }
}

impl Drop for DatabaseLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&DATABASE_LOCK_PATH);
    }
}

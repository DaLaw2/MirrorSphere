use crate::interface::database_ops::DatabaseOpsTrait;
use async_trait::async_trait;
use sqlx::SqlitePool;

#[derive(Clone, Debug)]
pub struct DatabaseOps {
    pool: SqlitePool,
}

#[async_trait]
impl DatabaseOpsTrait for DatabaseOps {
    fn new(pool: SqlitePool) -> Self {
        DatabaseOps { pool }
    }

    fn get_pool(&self) -> SqlitePool {
        self.pool.clone()
    }
}

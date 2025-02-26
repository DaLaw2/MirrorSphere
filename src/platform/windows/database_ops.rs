use sqlx::SqlitePool;
use crate::interface::database_ops::DatabaseOpsTrait;

#[derive(Clone, Debug)]
pub struct DatabaseOps {
    pool: SqlitePool,
}

impl DatabaseOpsTrait for DatabaseOps {
    fn new(pool: SqlitePool) -> Self {
        DatabaseOps { pool }
    }

    fn get_pool(&self) -> SqlitePool {
        self.pool.clone()
    }
}

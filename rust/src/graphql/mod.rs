use sqlx::SqlitePool;

pub mod data;
pub mod query;

pub struct Context {
    pub db: SqlitePool,
}

impl Context {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            db: pool,
        }
    }
}

impl juniper::Context for Context {}

use crate::db::Db;

#[derive(Debug, Clone, axum::extract::FromRef)]
pub struct AppState {
    pub db: Db,
}

impl AppState {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

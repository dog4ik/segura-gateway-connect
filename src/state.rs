use crate::{db::Db, gateway::SeguraGateway};

#[derive(Debug, Clone, axum::extract::FromRef)]
pub struct AppState {
    pub gate: SeguraGateway,
    pub db: Db,
}

impl AppState {
    pub fn new(db: Db) -> Self {
        let segura_gate = SeguraGateway::new();
        Self {
            gate: segura_gate,
            db,
        }
    }
}

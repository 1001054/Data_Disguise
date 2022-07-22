use sqlx::mysql::MySqlPool;

/// the state of the server
pub struct AppState {
    pub vault_db: MySqlPool,
    pub target_db: MySqlPool,
}
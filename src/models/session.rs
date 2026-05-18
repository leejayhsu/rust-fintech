use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    #[allow(dead_code)]
    #[serde(skip_serializing)]
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

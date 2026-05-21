use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Party {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub country_code: Option<String>,
    pub r#type: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePartyReq {
    #[validate(length(
        min = 1,
        max = 255,
        message = "name must be between 1 and 255 characters"
    ))]
    pub name: String,

    #[validate(email(message = "email must be a valid email address"))]
    pub email: Option<String>,

    #[validate(length(
        min = 1,
        max = 50,
        message = "phone must be between 1 and 50 characters"
    ))]
    pub phone: Option<String>,

    #[validate(length(
        min = 2,
        max = 2,
        message = "country_code must be a valid 2-letter ISO code"
    ))]
    pub country_code: Option<String>,

    #[validate(length(min = 1, max = 20))]
    pub r#type: String,

    #[validate(length(min = 1, max = 20))]
    pub role: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PartyResp {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub country_code: Option<String>,
    pub r#type: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Party> for PartyResp {
    fn from(p: Party) -> Self {
        Self {
            id: p.id,
            name: p.name,
            email: p.email,
            phone: p.phone,
            country_code: p.country_code,
            r#type: p.r#type,
            role: p.role,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

pub const VALID_PARTY_TYPES: &[&str] = &["individual", "business"];
pub const VALID_PARTY_ROLES: &[&str] = &["originator", "beneficiary", "counterparty"];
pub const DEFAULT_PARTY_ROLE: &str = "counterparty";
pub const ORIGINATOR_PARTY_ROLE: &str = "originator";

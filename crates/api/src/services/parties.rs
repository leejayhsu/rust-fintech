use sqlx::PgPool;

use crate::{
    errors::party_error::PartyError,
    models::party::{CreatePartyReq, DEFAULT_PARTY_ROLE, Party, PartyResp},
    utils,
};

pub async fn create(pool: &PgPool, req: CreatePartyReq) -> Result<PartyResp, PartyError> {
    let role = req.role.unwrap_or_else(|| DEFAULT_PARTY_ROLE.to_string());

    let party = sqlx::query_as!(
        Party,
        r#"
        INSERT INTO parties (id, name, email, phone, country_code, type, role, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, name, email, phone, country_code, type, role, created_at, updated_at
        "#,
        utils::generate_id("party"),
        req.name,
        req.email,
        req.phone,
        req.country_code,
        req.r#type,
        role,
        chrono::Utc::now(),
        chrono::Utc::now()
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            PartyError::PartyAlreadyExists
        }
        _ => PartyError::Database(e),
    })?;

    Ok(party.into())
}

pub async fn find_by_id(pool: &PgPool, id: &str) -> Result<PartyResp, PartyError> {
    let party = sqlx::query_as!(
        Party,
        r#"
        SELECT id, name, email, phone, country_code, type, role, created_at, updated_at
        FROM parties
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(PartyError::PartyNotFound)?;

    Ok(party.into())
}

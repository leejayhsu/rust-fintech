use serde_json::Value;
use sqlx::PgPool;

use crate::{
    errors::onboarding_error::OnboardingError,
    models::{
        onboarding::{
            AdminReviewClientOnboardingReq, ClientOnboarding, ClientOnboardingResp,
            CreateClientOnboardingReq, ONBOARDING_STATUS_APPROVED, ONBOARDING_STATUS_FAILED,
            ONBOARDING_STATUS_KYB_PENDING, ONBOARDING_STATUS_MANUAL_REVIEW_PENDING,
            ONBOARDING_STATUS_REJECTED,
        },
        party::ORIGINATOR_PARTY_ROLE,
    },
    utils,
};

const KYB_VENDOR_STATUS_PASSED: &str = "passed";
const KYB_VENDOR_STATUS_FAILED: &str = "failed";

pub async fn create(
    pool: &PgPool,
    submitted_by_user_id: &str,
    req: CreateClientOnboardingReq,
) -> Result<ClientOnboardingResp, OnboardingError> {
    let id = utils::generate_id("onb");
    let workflow_id = format!("client-onboarding-{id}");
    let now = chrono::Utc::now();

    let onboarding = sqlx::query_as!(
        ClientOnboarding,
        r#"
        INSERT INTO client_onboardings (
            id,
            submitted_by_user_id,
            company_name,
            company_email,
            phone,
            country_code,
            registration_number,
            address,
            status,
            temporal_workflow_id,
            created_at,
            updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING
            id,
            submitted_by_user_id,
            company_name,
            company_email,
            phone,
            country_code,
            registration_number,
            address,
            status,
            temporal_workflow_id,
            kyb_vendor_a_status,
            kyb_vendor_a_response,
            kyb_vendor_b_status,
            kyb_vendor_b_response,
            reviewed_by_user_id,
            reviewed_at,
            review_note,
            created_party_id,
            created_at,
            updated_at
        "#,
        id,
        submitted_by_user_id,
        req.company_name,
        req.company_email,
        req.phone,
        req.country_code,
        req.registration_number,
        req.address,
        ONBOARDING_STATUS_KYB_PENDING,
        workflow_id,
        now,
        now
    )
    .fetch_one(pool)
    .await?;

    Ok(onboarding.into())
}

pub async fn list_for_submitted_user(
    pool: &PgPool,
    submitted_by_user_id: &str,
) -> Result<Vec<ClientOnboardingResp>, OnboardingError> {
    let onboardings = sqlx::query_as!(
        ClientOnboarding,
        r#"
        SELECT
            id,
            submitted_by_user_id,
            company_name,
            company_email,
            phone,
            country_code,
            registration_number,
            address,
            status,
            temporal_workflow_id,
            kyb_vendor_a_status,
            kyb_vendor_a_response,
            kyb_vendor_b_status,
            kyb_vendor_b_response,
            reviewed_by_user_id,
            reviewed_at,
            review_note,
            created_party_id,
            created_at,
            updated_at
        FROM client_onboardings
        WHERE submitted_by_user_id = $1
        ORDER BY created_at DESC
        "#,
        submitted_by_user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(onboardings
        .into_iter()
        .map(ClientOnboardingResp::from)
        .collect())
}

pub async fn get_for_submitted_user(
    pool: &PgPool,
    submitted_by_user_id: &str,
    onboarding_id: &str,
) -> Result<ClientOnboardingResp, OnboardingError> {
    let onboarding = sqlx::query_as!(
        ClientOnboarding,
        r#"
        SELECT
            id,
            submitted_by_user_id,
            company_name,
            company_email,
            phone,
            country_code,
            registration_number,
            address,
            status,
            temporal_workflow_id,
            kyb_vendor_a_status,
            kyb_vendor_a_response,
            kyb_vendor_b_status,
            kyb_vendor_b_response,
            reviewed_by_user_id,
            reviewed_at,
            review_note,
            created_party_id,
            created_at,
            updated_at
        FROM client_onboardings
        WHERE id = $1 AND submitted_by_user_id = $2
        "#,
        onboarding_id,
        submitted_by_user_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(OnboardingError::NotFound)?;

    Ok(onboarding.into())
}

pub async fn list_for_admin(
    pool: &PgPool,
    status: Option<&str>,
) -> Result<Vec<ClientOnboardingResp>, OnboardingError> {
    let status = status.unwrap_or(ONBOARDING_STATUS_MANUAL_REVIEW_PENDING);
    let onboardings = sqlx::query_as!(
        ClientOnboarding,
        r#"
        SELECT
            id,
            submitted_by_user_id,
            company_name,
            company_email,
            phone,
            country_code,
            registration_number,
            address,
            status,
            temporal_workflow_id,
            kyb_vendor_a_status,
            kyb_vendor_a_response,
            kyb_vendor_b_status,
            kyb_vendor_b_response,
            reviewed_by_user_id,
            reviewed_at,
            review_note,
            created_party_id,
            created_at,
            updated_at
        FROM client_onboardings
        WHERE status = $1
        ORDER BY created_at ASC
        "#,
        status
    )
    .fetch_all(pool)
    .await?;

    Ok(onboardings
        .into_iter()
        .map(ClientOnboardingResp::from)
        .collect())
}

pub async fn get_for_admin(
    pool: &PgPool,
    onboarding_id: &str,
) -> Result<ClientOnboardingResp, OnboardingError> {
    let onboarding = find_by_id(pool, onboarding_id).await?;

    Ok(onboarding.into())
}

pub async fn record_kyb_results(
    pool: &PgPool,
    onboarding_id: &str,
    vendor_a: Value,
    vendor_b: Value,
) -> Result<ClientOnboardingResp, OnboardingError> {
    let vendor_a_passed = vendor_passes(&vendor_a);
    let vendor_b_passed = vendor_passes(&vendor_b);
    let passed = kyb_vendors_pass(&vendor_a, &vendor_b);
    let next_status = if passed {
        ONBOARDING_STATUS_MANUAL_REVIEW_PENDING
    } else {
        ONBOARDING_STATUS_REJECTED
    };
    let vendor_a_status = vendor_status(vendor_a_passed);
    let vendor_b_status = vendor_status(vendor_b_passed);

    let mut tx = pool.begin().await?;

    let current = sqlx::query_as!(
        ClientOnboarding,
        r#"
        SELECT
            id,
            submitted_by_user_id,
            company_name,
            company_email,
            phone,
            country_code,
            registration_number,
            address,
            status,
            temporal_workflow_id,
            kyb_vendor_a_status,
            kyb_vendor_a_response,
            kyb_vendor_b_status,
            kyb_vendor_b_response,
            reviewed_by_user_id,
            reviewed_at,
            review_note,
            created_party_id,
            created_at,
            updated_at
        FROM client_onboardings
        WHERE id = $1
        FOR UPDATE
        "#,
        onboarding_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(OnboardingError::NotFound)?;

    if current.status != ONBOARDING_STATUS_KYB_PENDING {
        if kyb_results_match(
            &current,
            vendor_a_status,
            &vendor_a,
            vendor_b_status,
            &vendor_b,
        ) {
            tx.commit().await?;
            return Ok(current.into());
        }

        return Err(OnboardingError::InvalidStatus);
    }

    let onboarding = sqlx::query_as!(
        ClientOnboarding,
        r#"
        UPDATE client_onboardings
        SET
            status = $2,
            kyb_vendor_a_status = $3,
            kyb_vendor_a_response = $4,
            kyb_vendor_b_status = $5,
            kyb_vendor_b_response = $6,
            updated_at = $7
        WHERE id = $1
        RETURNING
            id,
            submitted_by_user_id,
            company_name,
            company_email,
            phone,
            country_code,
            registration_number,
            address,
            status,
            temporal_workflow_id,
            kyb_vendor_a_status,
            kyb_vendor_a_response,
            kyb_vendor_b_status,
            kyb_vendor_b_response,
            reviewed_by_user_id,
            reviewed_at,
            review_note,
            created_party_id,
            created_at,
            updated_at
        "#,
        onboarding_id,
        next_status,
        vendor_a_status,
        vendor_a,
        vendor_b_status,
        vendor_b,
        chrono::Utc::now()
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(OnboardingError::NotFound)?;

    tx.commit().await?;

    Ok(onboarding.into())
}

pub async fn record_admin_decision(
    pool: &PgPool,
    onboarding_id: &str,
    admin_user_id: &str,
    req: AdminReviewClientOnboardingReq,
) -> Result<ClientOnboardingResp, OnboardingError> {
    let mut tx = pool.begin().await?;

    let current = sqlx::query_as!(
        ClientOnboarding,
        r#"
        SELECT
            id,
            submitted_by_user_id,
            company_name,
            company_email,
            phone,
            country_code,
            registration_number,
            address,
            status,
            temporal_workflow_id,
            kyb_vendor_a_status,
            kyb_vendor_a_response,
            kyb_vendor_b_status,
            kyb_vendor_b_response,
            reviewed_by_user_id,
            reviewed_at,
            review_note,
            created_party_id,
            created_at,
            updated_at
        FROM client_onboardings
        WHERE id = $1
        FOR UPDATE
        "#,
        onboarding_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(OnboardingError::NotFound)?;

    if admin_decision_matches_existing(&current, admin_user_id, &req) {
        tx.commit().await?;
        return Ok(current.into());
    }

    if current.status != ONBOARDING_STATUS_MANUAL_REVIEW_PENDING
        || current.reviewed_by_user_id.is_some()
    {
        return Err(OnboardingError::InvalidStatus);
    }

    let next_status = if req.approved {
        ONBOARDING_STATUS_MANUAL_REVIEW_PENDING
    } else {
        ONBOARDING_STATUS_REJECTED
    };

    let onboarding = sqlx::query_as!(
        ClientOnboarding,
        r#"
        UPDATE client_onboardings
        SET
            status = $2,
            reviewed_by_user_id = $3,
            reviewed_at = $4,
            review_note = $5,
            updated_at = $4
        WHERE id = $1
        RETURNING
            id,
            submitted_by_user_id,
            company_name,
            company_email,
            phone,
            country_code,
            registration_number,
            address,
            status,
            temporal_workflow_id,
            kyb_vendor_a_status,
            kyb_vendor_a_response,
            kyb_vendor_b_status,
            kyb_vendor_b_response,
            reviewed_by_user_id,
            reviewed_at,
            review_note,
            created_party_id,
            created_at,
            updated_at
        "#,
        onboarding_id,
        next_status,
        admin_user_id,
        chrono::Utc::now(),
        req.note
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(OnboardingError::NotFound)?;

    tx.commit().await?;

    Ok(onboarding.into())
}

pub async fn complete_with_originator_party(
    pool: &PgPool,
    onboarding_id: &str,
) -> Result<ClientOnboardingResp, OnboardingError> {
    let mut tx = pool.begin().await?;

    let onboarding = sqlx::query_as!(
        ClientOnboarding,
        r#"
        SELECT
            id,
            submitted_by_user_id,
            company_name,
            company_email,
            phone,
            country_code,
            registration_number,
            address,
            status,
            temporal_workflow_id,
            kyb_vendor_a_status,
            kyb_vendor_a_response,
            kyb_vendor_b_status,
            kyb_vendor_b_response,
            reviewed_by_user_id,
            reviewed_at,
            review_note,
            created_party_id,
            created_at,
            updated_at
        FROM client_onboardings
        WHERE id = $1
        FOR UPDATE
        "#,
        onboarding_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(OnboardingError::NotFound)?;

    if onboarding.status == ONBOARDING_STATUS_APPROVED && onboarding.created_party_id.is_some() {
        tx.commit().await?;
        return Ok(onboarding.into());
    }

    if onboarding.status != ONBOARDING_STATUS_MANUAL_REVIEW_PENDING
        || onboarding.reviewed_by_user_id.is_none()
    {
        return Err(OnboardingError::InvalidStatus);
    }

    let party_id = utils::generate_id("party");
    let now = chrono::Utc::now();

    sqlx::query!(
        r#"
        INSERT INTO parties (id, name, email, phone, country_code, type, role, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, 'business', $6, $7, $7)
        "#,
        party_id,
        onboarding.company_name,
        onboarding.company_email,
        onboarding.phone,
        onboarding.country_code,
        ORIGINATOR_PARTY_ROLE,
        now
    )
    .execute(&mut *tx)
    .await?;

    let updated = sqlx::query_as!(
        ClientOnboarding,
        r#"
        UPDATE client_onboardings
        SET
            status = $2,
            created_party_id = $3,
            updated_at = $4
        WHERE id = $1
        RETURNING
            id,
            submitted_by_user_id,
            company_name,
            company_email,
            phone,
            country_code,
            registration_number,
            address,
            status,
            temporal_workflow_id,
            kyb_vendor_a_status,
            kyb_vendor_a_response,
            kyb_vendor_b_status,
            kyb_vendor_b_response,
            reviewed_by_user_id,
            reviewed_at,
            review_note,
            created_party_id,
            created_at,
            updated_at
        "#,
        onboarding_id,
        ONBOARDING_STATUS_APPROVED,
        party_id,
        now
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(updated.into())
}

pub async fn set_failed(
    pool: &PgPool,
    onboarding_id: &str,
) -> Result<ClientOnboardingResp, OnboardingError> {
    let onboarding = sqlx::query_as!(
        ClientOnboarding,
        r#"
        UPDATE client_onboardings
        SET status = $2, updated_at = $3
        WHERE id = $1
        RETURNING
            id,
            submitted_by_user_id,
            company_name,
            company_email,
            phone,
            country_code,
            registration_number,
            address,
            status,
            temporal_workflow_id,
            kyb_vendor_a_status,
            kyb_vendor_a_response,
            kyb_vendor_b_status,
            kyb_vendor_b_response,
            reviewed_by_user_id,
            reviewed_at,
            review_note,
            created_party_id,
            created_at,
            updated_at
        "#,
        onboarding_id,
        ONBOARDING_STATUS_FAILED,
        chrono::Utc::now()
    )
    .fetch_optional(pool)
    .await?
    .ok_or(OnboardingError::NotFound)?;

    Ok(onboarding.into())
}

pub fn kyb_vendors_pass(vendor_a: &Value, vendor_b: &Value) -> bool {
    vendor_passes(vendor_a) && vendor_passes(vendor_b)
}

async fn find_by_id(
    pool: &PgPool,
    onboarding_id: &str,
) -> Result<ClientOnboarding, OnboardingError> {
    let onboarding = sqlx::query_as!(
        ClientOnboarding,
        r#"
        SELECT
            id,
            submitted_by_user_id,
            company_name,
            company_email,
            phone,
            country_code,
            registration_number,
            address,
            status,
            temporal_workflow_id,
            kyb_vendor_a_status,
            kyb_vendor_a_response,
            kyb_vendor_b_status,
            kyb_vendor_b_response,
            reviewed_by_user_id,
            reviewed_at,
            review_note,
            created_party_id,
            created_at,
            updated_at
        FROM client_onboardings
        WHERE id = $1
        "#,
        onboarding_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(OnboardingError::NotFound)?;

    Ok(onboarding)
}

fn vendor_passes(vendor: &Value) -> bool {
    vendor.get("companyExists").and_then(Value::as_bool) == Some(true)
        && vendor.get("sanctioned").and_then(Value::as_bool) == Some(false)
        && vendor.get("ofacListed").and_then(Value::as_bool) == Some(false)
}

fn vendor_status(passed: bool) -> &'static str {
    if passed {
        KYB_VENDOR_STATUS_PASSED
    } else {
        KYB_VENDOR_STATUS_FAILED
    }
}

fn kyb_results_match(
    onboarding: &ClientOnboarding,
    vendor_a_status: &str,
    vendor_a: &Value,
    vendor_b_status: &str,
    vendor_b: &Value,
) -> bool {
    is_valid_kyb_duplicate_status(&onboarding.status)
        && onboarding.kyb_vendor_a_status.as_deref() == Some(vendor_a_status)
        && onboarding.kyb_vendor_a_response.as_ref() == Some(vendor_a)
        && onboarding.kyb_vendor_b_status.as_deref() == Some(vendor_b_status)
        && onboarding.kyb_vendor_b_response.as_ref() == Some(vendor_b)
}

fn is_valid_kyb_duplicate_status(status: &str) -> bool {
    matches!(
        status,
        ONBOARDING_STATUS_MANUAL_REVIEW_PENDING
            | ONBOARDING_STATUS_REJECTED
            | ONBOARDING_STATUS_APPROVED
    )
}

fn admin_decision_matches_existing(
    onboarding: &ClientOnboarding,
    admin_user_id: &str,
    req: &AdminReviewClientOnboardingReq,
) -> bool {
    let same_reviewer = onboarding.reviewed_by_user_id.as_deref() == Some(admin_user_id);
    let same_note = onboarding.review_note.as_deref() == req.note.as_deref();

    if !same_reviewer || !same_note {
        return false;
    }

    if req.approved {
        onboarding.status == ONBOARDING_STATUS_MANUAL_REVIEW_PENDING
            || onboarding.status == ONBOARDING_STATUS_APPROVED
    } else {
        onboarding.status == ONBOARDING_STATUS_REJECTED
    }
}

#[cfg(test)]
mod onboarding_service {
    use chrono::Utc;
    use serde_json::json;

    use crate::models::onboarding::{
        ClientOnboarding, ONBOARDING_STATUS_APPROVED, ONBOARDING_STATUS_FAILED,
        ONBOARDING_STATUS_MANUAL_REVIEW_PENDING,
    };

    #[test]
    fn kyb_vendors_pass_returns_true_when_both_vendor_flags_are_pass_values() {
        let vendor_a = json!({
            "companyExists": true,
            "sanctioned": false,
            "ofacListed": false
        });
        let vendor_b = json!({
            "companyExists": true,
            "sanctioned": false,
            "ofacListed": false
        });

        assert!(super::kyb_vendors_pass(&vendor_a, &vendor_b));
    }

    #[test]
    fn kyb_vendors_pass_returns_false_when_a_required_flag_is_missing() {
        let vendor_a = json!({
            "companyExists": true,
            "sanctioned": false
        });
        let vendor_b = json!({
            "companyExists": true,
            "sanctioned": false,
            "ofacListed": false
        });

        assert!(!super::kyb_vendors_pass(&vendor_a, &vendor_b));
    }

    #[test]
    fn kyb_vendors_pass_returns_false_when_a_required_flag_has_wrong_type() {
        let vendor_a = json!({
            "companyExists": true,
            "sanctioned": "false",
            "ofacListed": false
        });
        let vendor_b = json!({
            "companyExists": true,
            "sanctioned": false,
            "ofacListed": false
        });

        assert!(!super::kyb_vendors_pass(&vendor_a, &vendor_b));
    }

    #[test]
    fn kyb_results_match_accepts_duplicate_after_approval_when_payloads_match() {
        let vendor_a = json!({
            "companyExists": true,
            "sanctioned": false,
            "ofacListed": false
        });
        let vendor_b = json!({
            "companyExists": true,
            "sanctioned": false,
            "ofacListed": false
        });
        let onboarding = onboarding_with_kyb(
            ONBOARDING_STATUS_APPROVED,
            Some("passed"),
            Some(vendor_a.clone()),
            Some("passed"),
            Some(vendor_b.clone()),
        );

        assert!(super::kyb_results_match(
            &onboarding,
            "passed",
            &vendor_a,
            "passed",
            &vendor_b
        ));
    }

    #[test]
    fn kyb_results_match_rejects_duplicate_when_payloads_differ() {
        let stored_vendor_a = json!({
            "companyExists": true,
            "sanctioned": false,
            "ofacListed": false
        });
        let incoming_vendor_a = json!({
            "companyExists": true,
            "sanctioned": true,
            "ofacListed": false
        });
        let vendor_b = json!({
            "companyExists": true,
            "sanctioned": false,
            "ofacListed": false
        });
        let onboarding = onboarding_with_kyb(
            ONBOARDING_STATUS_MANUAL_REVIEW_PENDING,
            Some("passed"),
            Some(stored_vendor_a),
            Some("passed"),
            Some(vendor_b.clone()),
        );

        assert!(!super::kyb_results_match(
            &onboarding,
            "failed",
            &incoming_vendor_a,
            "passed",
            &vendor_b
        ));
    }

    #[test]
    fn kyb_results_match_rejects_duplicate_in_invalid_later_status() {
        let vendor_a = json!({
            "companyExists": true,
            "sanctioned": false,
            "ofacListed": false
        });
        let vendor_b = json!({
            "companyExists": true,
            "sanctioned": false,
            "ofacListed": false
        });
        let onboarding = onboarding_with_kyb(
            ONBOARDING_STATUS_FAILED,
            Some("passed"),
            Some(vendor_a.clone()),
            Some("passed"),
            Some(vendor_b.clone()),
        );

        assert!(!super::kyb_results_match(
            &onboarding,
            "passed",
            &vendor_a,
            "passed",
            &vendor_b
        ));
    }

    fn onboarding_with_kyb(
        status: &str,
        vendor_a_status: Option<&str>,
        vendor_a_response: Option<serde_json::Value>,
        vendor_b_status: Option<&str>,
        vendor_b_response: Option<serde_json::Value>,
    ) -> ClientOnboarding {
        let now = Utc::now();

        ClientOnboarding {
            id: "onb_test".to_string(),
            submitted_by_user_id: "usr_test".to_string(),
            company_name: "Acme".to_string(),
            company_email: None,
            phone: None,
            country_code: "US".to_string(),
            registration_number: None,
            address: None,
            status: status.to_string(),
            temporal_workflow_id: Some("client-onboarding-onb_test".to_string()),
            kyb_vendor_a_status: vendor_a_status.map(str::to_string),
            kyb_vendor_a_response: vendor_a_response,
            kyb_vendor_b_status: vendor_b_status.map(str::to_string),
            kyb_vendor_b_response: vendor_b_response,
            reviewed_by_user_id: None,
            reviewed_at: None,
            review_note: None,
            created_party_id: None,
            created_at: now,
            updated_at: now,
        }
    }
}

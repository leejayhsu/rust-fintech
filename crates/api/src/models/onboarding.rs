use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;

pub const ONBOARDING_STATUS_DRAFT: &str = "draft";
pub const ONBOARDING_STATUS_KYB_PENDING: &str = "kyb_pending";
pub const ONBOARDING_STATUS_MANUAL_REVIEW_PENDING: &str = "manual_review_pending";
pub const ONBOARDING_STATUS_APPROVED: &str = "approved";
pub const ONBOARDING_STATUS_REJECTED: &str = "rejected";
pub const ONBOARDING_STATUS_FAILED: &str = "failed";

pub const VALID_ONBOARDING_STATUSES: &[&str] = &[
    ONBOARDING_STATUS_DRAFT,
    ONBOARDING_STATUS_KYB_PENDING,
    ONBOARDING_STATUS_MANUAL_REVIEW_PENDING,
    ONBOARDING_STATUS_APPROVED,
    ONBOARDING_STATUS_REJECTED,
    ONBOARDING_STATUS_FAILED,
];

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ClientOnboarding {
    pub id: String,
    pub submitted_by_user_id: String,
    pub company_name: String,
    pub company_email: Option<String>,
    pub phone: Option<String>,
    pub country_code: String,
    pub registration_number: Option<String>,
    pub address: Option<String>,
    pub status: String,
    pub temporal_workflow_id: Option<String>,
    pub kyb_vendor_a_status: Option<String>,
    pub kyb_vendor_a_response: Option<Value>,
    pub kyb_vendor_b_status: Option<String>,
    pub kyb_vendor_b_response: Option<Value>,
    pub reviewed_by_user_id: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_note: Option<String>,
    pub created_party_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ClientOnboardingResp {
    pub id: String,
    pub submitted_by_user_id: String,
    pub company_name: String,
    pub company_email: Option<String>,
    pub phone: Option<String>,
    pub country_code: String,
    pub registration_number: Option<String>,
    pub address: Option<String>,
    pub status: String,
    pub temporal_workflow_id: Option<String>,
    pub kyb_vendor_a_status: Option<String>,
    pub kyb_vendor_a_response: Option<Value>,
    pub kyb_vendor_b_status: Option<String>,
    pub kyb_vendor_b_response: Option<Value>,
    pub reviewed_by_user_id: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_note: Option<String>,
    pub created_party_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ClientOnboarding> for ClientOnboardingResp {
    fn from(onboarding: ClientOnboarding) -> Self {
        Self {
            id: onboarding.id,
            submitted_by_user_id: onboarding.submitted_by_user_id,
            company_name: onboarding.company_name,
            company_email: onboarding.company_email,
            phone: onboarding.phone,
            country_code: onboarding.country_code,
            registration_number: onboarding.registration_number,
            address: onboarding.address,
            status: onboarding.status,
            temporal_workflow_id: onboarding.temporal_workflow_id,
            kyb_vendor_a_status: onboarding.kyb_vendor_a_status,
            kyb_vendor_a_response: onboarding.kyb_vendor_a_response,
            kyb_vendor_b_status: onboarding.kyb_vendor_b_status,
            kyb_vendor_b_response: onboarding.kyb_vendor_b_response,
            reviewed_by_user_id: onboarding.reviewed_by_user_id,
            reviewed_at: onboarding.reviewed_at,
            review_note: onboarding.review_note,
            created_party_id: onboarding.created_party_id,
            created_at: onboarding.created_at,
            updated_at: onboarding.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateClientOnboardingReq {
    #[validate(length(min = 1, max = 255))]
    pub company_name: String,

    #[validate(email)]
    pub company_email: Option<String>,

    #[validate(length(min = 1, max = 50))]
    pub phone: Option<String>,

    #[validate(length(min = 2, max = 2))]
    pub country_code: String,

    #[validate(length(min = 1, max = 100))]
    pub registration_number: Option<String>,

    #[validate(length(min = 1, max = 500))]
    pub address: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct AdminReviewClientOnboardingReq {
    pub approved: bool,

    #[validate(length(max = 1000))]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordKybResultsReq {
    pub vendor_a: Value,
    pub vendor_b: Value,
}

#[derive(Debug, Deserialize)]
pub struct ListClientOnboardingsQuery {
    pub status: Option<String>,
}

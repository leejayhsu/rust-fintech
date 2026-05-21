use serde::{Deserialize, Serialize};

pub const TASK_QUEUE: &str = "client-onboarding";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientOnboardingWorkflowInput {
    pub onboarding_id: String,
    pub workflow_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FakeKybVendorResult {
    pub vendor: String,
    pub company_exists: bool,
    pub sanctioned: bool,
    pub ofac_listed: bool,
    pub reference_id: String,
    pub checked_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminReviewSignal {
    pub approved: bool,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub error: Option<ApiError>,
    pub data: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientOnboardingResp {
    pub id: String,
    pub status: String,
    pub created_party_id: Option<String>,
}

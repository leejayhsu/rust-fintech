use reqwest::Client;
use serde::Serialize;

use crate::types::{ApiResponse, ClientOnboardingResp, FakeKybVendorResult};

const DEFAULT_API_BASE_URL: &str = "http://localhost:3000";

#[derive(Debug, thiserror::Error)]
pub enum ApiActivityError {
    #[error("INTERNAL_API_TOKEN must be set for Temporal worker callbacks")]
    MissingToken,
    #[error("http request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("api request failed: {0}")]
    ApiFailure(String),
}

pub async fn record_kyb_results(
    onboarding_id: &str,
    vendor_a: FakeKybVendorResult,
    vendor_b: FakeKybVendorResult,
) -> Result<ClientOnboardingResp, ApiActivityError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Body {
        vendor_a: FakeKybVendorResult,
        vendor_b: FakeKybVendorResult,
    }

    internal_api_request(
        &format!("/api/v1/internal/onboardings/{onboarding_id}/kyb-results"),
        Body { vendor_a, vendor_b },
    )
    .await
}

pub async fn complete_onboarding(
    onboarding_id: &str,
) -> Result<ClientOnboardingResp, ApiActivityError> {
    internal_api_request::<_, ClientOnboardingResp>(
        &format!("/api/v1/internal/onboardings/{onboarding_id}/complete"),
        serde_json::json!({}),
    )
    .await
}

async fn internal_api_request<B: Serialize, T: serde::de::DeserializeOwned>(
    path: &str,
    body: B,
) -> Result<T, ApiActivityError> {
    let token = std::env::var("INTERNAL_API_TOKEN")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .ok_or(ApiActivityError::MissingToken)?;

    let response = Client::new()
        .post(format!("{}{}", api_base_url(), path))
        .bearer_auth(token)
        .json(&body)
        .send()
        .await?;

    let status = response.status();
    let payload: ApiResponse<T> = response.json().await?;

    if !status.is_success() || !payload.success || payload.data.is_none() {
        return Err(ApiActivityError::ApiFailure(
            payload
                .error
                .map(|e| e.desc)
                .unwrap_or_else(|| format!("API request failed with {status}")),
        ));
    }

    match payload.data {
        Some(data) => Ok(data),
        None => Err(ApiActivityError::ApiFailure("API response data missing".to_string())),
    }
}

fn api_base_url() -> String {
    std::env::var("API_BASE_URL")
        .unwrap_or_else(|_| DEFAULT_API_BASE_URL.to_string())
        .trim_end_matches('/')
        .to_string()
}

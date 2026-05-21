use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::models::onboarding::AdminReviewClientOnboardingReq;

const DEFAULT_ONBOARDING_WORKER_URL: &str = "http://localhost:4100";

#[derive(Debug, thiserror::Error)]
pub enum OnboardingWorkerClientError {
    #[error("worker request failed: {0}")]
    Request(#[from] reqwest::Error),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StartClientOnboardingWorkflowReq<'a> {
    onboarding_id: &'a str,
    workflow_id: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReviewClientOnboardingWorkflowReq<'a> {
    approved: bool,
    note: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct WorkerAck {}

pub async fn start_client_onboarding(
    onboarding_id: &str,
    workflow_id: &str,
) -> Result<(), OnboardingWorkerClientError> {
    let url = format!(
        "{}/internal/workflows/client-onboarding/start",
        worker_url()
    );
    let body = StartClientOnboardingWorkflowReq {
        onboarding_id,
        workflow_id,
    };

    http_client()?
        .post(url)
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<WorkerAck>()
        .await?;

    Ok(())
}

/// Signals the worker that an admin review decision was recorded.
///
/// Contract for the TypeScript bridge: this call may be retried for the same
/// onboarding and same decision after an API timeout or worker/network failure.
/// The bridge should treat same-decision duplicate signals as success whenever
/// possible, including workflows that are already signaled, completed, or
/// otherwise closed after applying that same decision.
pub async fn signal_admin_review(
    onboarding_id: &str,
    req: &AdminReviewClientOnboardingReq,
) -> Result<(), OnboardingWorkerClientError> {
    let url = format!(
        "{}/internal/workflows/client-onboarding/{onboarding_id}/review",
        worker_url()
    );
    let body = ReviewClientOnboardingWorkflowReq {
        approved: req.approved,
        note: req.note.as_deref(),
    };

    http_client()?
        .post(url)
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<WorkerAck>()
        .await?;

    Ok(())
}

fn worker_url() -> String {
    std::env::var("ONBOARDING_WORKER_URL")
        .unwrap_or_else(|_| DEFAULT_ONBOARDING_WORKER_URL.to_string())
        .trim_end_matches('/')
        .to_string()
}

fn http_client() -> Result<Client, OnboardingWorkerClientError> {
    Ok(Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(5))
        .build()?)
}

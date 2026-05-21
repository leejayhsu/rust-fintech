mod activities;
mod api;
mod types;

use axum::{Json, Router, extract::{Path, State}, http::StatusCode, routing::post};
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::error;

type Decisions = Arc<RwLock<HashMap<String, types::AdminReviewSignal>>>;

#[derive(Clone)]
struct AppState {
    decisions: Decisions,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartReq {
    onboarding_id: String,
    workflow_id: String,
}

#[derive(Deserialize)]
struct ReviewReq {
    approved: bool,
    note: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let app = Router::new()
        .route("/internal/workflows/client-onboarding/start", post(start))
        .route("/internal/workflows/client-onboarding/{onboarding_id}/review", post(review))
        .with_state(AppState { decisions: Arc::new(RwLock::new(HashMap::new())) });

    let port = std::env::var("ONBOARDING_WORKER_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(4100);
    match tokio::net::TcpListener::bind(("0.0.0.0", port)).await {
        Ok(listener) => {
            tracing::info!("temporal onboarding bridge listening on {port}");
            if let Err(e) = axum::serve(listener, app).await {
                error!("server exited with error: {e}");
            }
        }
        Err(e) => error!("failed to bind temporal worker bridge on port {port}: {e}"),
    }
}

async fn start(State(state): State<AppState>, Json(req): Json<StartReq>) -> StatusCode {
    let decisions = state.decisions.clone();
    tokio::spawn(async move {
        if let Err(e) = run_workflow(req, decisions).await {
            error!("workflow failed: {e}");
        }
    });
    StatusCode::OK
}

async fn review(
    State(state): State<AppState>,
    Path(onboarding_id): Path<String>,
    Json(req): Json<ReviewReq>,
) -> StatusCode {
    state.decisions.write().await.insert(onboarding_id, types::AdminReviewSignal { approved: req.approved, note: req.note });
    StatusCode::OK
}

async fn run_workflow(req: StartReq, decisions: Decisions) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let input = types::ClientOnboardingWorkflowInput { onboarding_id: req.onboarding_id.clone(), workflow_id: req.workflow_id };
    let (vendor_a, vendor_b) = tokio::join!(activities::run_vendor_a(&input), activities::run_vendor_b(&input));

    let kyb = api::record_kyb_results(&req.onboarding_id, vendor_a, vendor_b).await?;
    if kyb.status == "rejected" {
        return Ok(());
    }

    loop {
        if let Some(decision) = decisions.write().await.remove(&req.onboarding_id) {
            if !decision.approved {
                return Ok(());
            }
            let _ = api::complete_onboarding(&req.onboarding_id).await?;
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
}

use chrono::Utc;
use tokio::time::{Duration, sleep};

use crate::types::{ClientOnboardingWorkflowInput, FakeKybVendorResult};

const FAKE_VENDOR_DELAY_MS: u64 = 10_000;

pub async fn run_vendor_a(input: &ClientOnboardingWorkflowInput) -> FakeKybVendorResult {
    sleep(Duration::from_millis(FAKE_VENDOR_DELAY_MS)).await;
    fake_result("vendor_a", &input.onboarding_id)
}

pub async fn run_vendor_b(input: &ClientOnboardingWorkflowInput) -> FakeKybVendorResult {
    sleep(Duration::from_millis(FAKE_VENDOR_DELAY_MS)).await;
    fake_result("vendor_b", &input.onboarding_id)
}

fn fake_result(vendor: &str, onboarding_id: &str) -> FakeKybVendorResult {
    FakeKybVendorResult {
        vendor: vendor.to_string(),
        company_exists: true,
        sanctioned: false,
        ofac_listed: false,
        reference_id: format!("fake-{vendor}-{onboarding_id}"),
        checked_at: Utc::now().to_rfc3339(),
    }
}

import type { ClientOnboardingWorkflowInput, FakeKybVendorResult } from "../types";

const FAKE_VENDOR_DELAY_MS = 10_000;

export async function runVendorA(
  input: ClientOnboardingWorkflowInput,
): Promise<FakeKybVendorResult> {
  await delay(FAKE_VENDOR_DELAY_MS);
  return fakeResult("vendor_a", input.onboardingId);
}

export async function runVendorB(
  input: ClientOnboardingWorkflowInput,
): Promise<FakeKybVendorResult> {
  await delay(FAKE_VENDOR_DELAY_MS);
  return fakeResult("vendor_b", input.onboardingId);
}

function fakeResult(
  vendor: FakeKybVendorResult["vendor"],
  onboardingId: string,
): FakeKybVendorResult {
  return {
    vendor,
    companyExists: true,
    sanctioned: false,
    ofacListed: false,
    referenceId: `fake-${vendor}-${onboardingId}`,
    checkedAt: new Date().toISOString(),
  };
}

function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

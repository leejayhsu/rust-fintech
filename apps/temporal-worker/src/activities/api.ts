import type {
  ApiResponse,
  ClientOnboardingResp,
  FakeKybVendorResult,
} from "../types";

const DEFAULT_API_BASE_URL = "http://localhost:3000";

export async function recordKybResults(input: {
  onboardingId: string;
  vendorA: FakeKybVendorResult;
  vendorB: FakeKybVendorResult;
}): Promise<ClientOnboardingResp> {
  return internalApiRequest<ClientOnboardingResp>(
    `/api/v1/internal/onboardings/${input.onboardingId}/kyb-results`,
    {
      vendorA: input.vendorA,
      vendorB: input.vendorB,
    },
  );
}

export async function completeOnboarding(input: {
  onboardingId: string;
}): Promise<ClientOnboardingResp> {
  return internalApiRequest<ClientOnboardingResp>(
    `/api/v1/internal/onboardings/${input.onboardingId}/complete`,
    {},
  );
}

async function internalApiRequest<T>(path: string, body: unknown): Promise<T> {
  const token = process.env.INTERNAL_API_TOKEN?.trim();
  if (!token) {
    throw new Error("INTERNAL_API_TOKEN must be set for Temporal worker callbacks");
  }

  const response = await fetch(`${apiBaseUrl()}${path}`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(body),
  });

  const payload = (await response.json()) as ApiResponse<T>;
  if (!response.ok || !payload.success || payload.data === null) {
    throw new Error(payload.error?.desc ?? `API request failed with ${response.status}`);
  }

  return payload.data;
}

function apiBaseUrl() {
  return (process.env.API_BASE_URL ?? DEFAULT_API_BASE_URL).replace(/\/$/, "");
}

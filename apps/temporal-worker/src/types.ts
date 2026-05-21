export const TASK_QUEUE = "client-onboarding";

export type ClientOnboardingWorkflowInput = {
  onboardingId: string;
  workflowId: string;
};

export type FakeKybVendorResult = {
  vendor: "vendor_a" | "vendor_b";
  companyExists: boolean;
  sanctioned: boolean;
  ofacListed: boolean;
  referenceId: string;
  checkedAt: string;
};

export type AdminReviewSignal = {
  approved: boolean;
  note?: string | null;
};

export type ApiResponse<T> = {
  success: boolean;
  error: { code: string; desc: string } | null;
  data: T | null;
};

export type ClientOnboardingResp = {
  id: string;
  status: "draft" | "kyb_pending" | "manual_review_pending" | "approved" | "rejected" | "failed";
  created_party_id?: string | null;
};

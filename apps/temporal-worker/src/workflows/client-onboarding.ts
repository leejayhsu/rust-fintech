import {
  condition,
  defineSignal,
  proxyActivities,
  setHandler,
} from "@temporalio/workflow";

import type * as apiActivities from "../activities/api";
import type * as kybActivities from "../activities/kyb";
import type {
  AdminReviewSignal,
  ClientOnboardingWorkflowInput,
} from "../types";

const kyb = proxyActivities<typeof kybActivities>({
  startToCloseTimeout: "30 seconds",
  retry: {
    maximumAttempts: 3,
  },
});

const api = proxyActivities<typeof apiActivities>({
  startToCloseTimeout: "15 seconds",
  retry: {
    maximumAttempts: 5,
  },
});

export const reviewSignal =
  defineSignal<[AdminReviewSignal]>("adminReview");

export async function clientOnboardingWorkflow(
  input: ClientOnboardingWorkflowInput,
) {
  let approvalDecision: AdminReviewSignal | undefined;

  setHandler(reviewSignal, (decision) => {
    approvalDecision = decision;
  });

  const [vendorA, vendorB] = await Promise.all([
    kyb.runVendorA(input),
    kyb.runVendorB(input),
  ]);

  const kybResult = await api.recordKybResults({
    onboardingId: input.onboardingId,
    vendorA,
    vendorB,
  });

  if (kybResult.status === "rejected") {
    return { status: "rejected" as const };
  }

  await condition(() => approvalDecision !== undefined);

  if (!approvalDecision?.approved) {
    return { status: "rejected" as const };
  }

  const completed = await api.completeOnboarding({
    onboardingId: input.onboardingId,
  });

  return {
    status: completed.status,
    partyId: completed.created_party_id,
  };
}

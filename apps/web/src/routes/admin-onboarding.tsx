import {
  getCurrentUser,
  listAdminOnboardings,
  reviewClientOnboarding,
  type ClientOnboarding,
} from "@rust-fintech/api-client";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Check, ShieldCheck, X } from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export function AdminOnboardingRoute() {
  const queryClient = useQueryClient();
  const [selected, setSelected] = useState<ClientOnboarding | null>(null);
  const [note, setNote] = useState("");

  const currentUser = useQuery({
    queryKey: ["current-user"],
    queryFn: async () => {
      const response = await getCurrentUser();
      return response.data.data;
    },
  });

  const onboardings = useQuery({
    queryKey: ["admin-onboardings", "manual_review_pending"],
    enabled: currentUser.data?.role === "admin",
    queryFn: async () => {
      const response = await listAdminOnboardings({
        status: "manual_review_pending",
      });
      return response.data.data ?? [];
    },
  });

  const reviewMutation = useMutation({
    mutationFn: async (input: { approved: boolean }) => {
      if (!selected) {
        throw new Error("Select an onboarding first");
      }
      const response = await reviewClientOnboarding(selected.id, {
        approved: input.approved,
        note: note.trim() || undefined,
      });
      return response.data.data;
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["admin-onboardings"] });
      setSelected(null);
      setNote("");
    },
  });

  const isAdmin = currentUser.data?.role === "admin";

  return (
    <section className="workspace-page">
      <header className="workspace-header">
        <div>
          <p className="eyebrow">Backoffice</p>
          <h1>Onboarding approvals</h1>
        </div>
        <a className="text-link" href="/dashboard">
          Dashboard
        </a>
      </header>

      {!isAdmin ? (
        <div className="empty-state">
          <ShieldCheck aria-hidden="true" />
          <strong>Admin access required</strong>
        </div>
      ) : (
        <div className="admin-layout">
          <div className="admin-list">
            {(onboardings.data ?? []).map((onboarding) => (
              <button
                key={onboarding.id}
                className="admin-row"
                data-active={selected?.id === onboarding.id ? "true" : undefined}
                type="button"
                onClick={() => setSelected(onboarding)}
              >
                <strong>{onboarding.company_name}</strong>
                <span>{onboarding.country_code}</span>
                <span>{onboarding.kyb_vendor_a_status ?? "vendor a pending"}</span>
                <span>{onboarding.kyb_vendor_b_status ?? "vendor b pending"}</span>
              </button>
            ))}
            {onboardings.data?.length === 0 ? (
              <div className="empty-state">No onboardings need review.</div>
            ) : null}
          </div>

          <aside className="review-panel">
            {selected ? (
              <>
                <h2>{selected.company_name}</h2>
                <dl className="review-list compact">
                  <div>
                    <dt>Status</dt>
                    <dd>{selected.status}</dd>
                  </div>
                  <div>
                    <dt>Submitted by</dt>
                    <dd>{selected.submitted_by_user_id}</dd>
                  </div>
                  <div>
                    <dt>Vendor A</dt>
                    <dd>{selected.kyb_vendor_a_status ?? "Pending"}</dd>
                  </div>
                  <div>
                    <dt>Vendor B</dt>
                    <dd>{selected.kyb_vendor_b_status ?? "Pending"}</dd>
                  </div>
                </dl>
                <Input
                  aria-label="Review note"
                  placeholder="Review note"
                  value={note}
                  onChange={(event) => setNote(event.target.value)}
                />
                {reviewMutation.error ? (
                  <p className="field-error">{reviewMutation.error.message}</p>
                ) : null}
                <div className="review-actions">
                  <Button
                    disabled={reviewMutation.isPending}
                    type="button"
                    onClick={() => reviewMutation.mutate({ approved: true })}
                  >
                    <Check aria-hidden="true" />
                    Approve
                  </Button>
                  <Button
                    disabled={reviewMutation.isPending}
                    type="button"
                    variant="outline"
                    onClick={() => reviewMutation.mutate({ approved: false })}
                  >
                    <X aria-hidden="true" />
                    Reject
                  </Button>
                </div>
              </>
            ) : (
              <div className="empty-state">Select a company to review.</div>
            )}
          </aside>
        </div>
      )}
    </section>
  );
}

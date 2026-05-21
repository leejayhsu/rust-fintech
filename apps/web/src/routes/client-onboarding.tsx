import {
  createClientOnboarding,
  getMyClientOnboarding,
  type ClientOnboarding,
  type CreateClientOnboardingReq,
} from "@rust-fintech/api-client";
import { useForm } from "@tanstack/react-form";
import { useMutation, useQuery } from "@tanstack/react-query";
import { ArrowLeft, ArrowRight, Building2, CheckCircle2, Clock3, Send } from "lucide-react";
import type React from "react";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

type WizardValues = {
  company_name: string;
  country_code: string;
  registration_number: string;
  company_email: string;
  phone: string;
  address: string;
};

const steps = ["Company", "Contact", "Review"] as const;

export function ClientOnboardingRoute() {
  const [step, setStep] = useState(0);
  const [submittedId, setSubmittedId] = useState<string | null>(null);

  const onboardingQuery = useQuery({
    queryKey: ["client-onboarding", submittedId],
    enabled: submittedId !== null,
    queryFn: async () => unwrapOnboarding(await getMyClientOnboarding(submittedId ?? "")),
    refetchInterval: (query) =>
      query.state.data?.status === "kyb_pending" ||
      query.state.data?.status === "manual_review_pending"
        ? 5000
        : false,
  });

  const createMutation = useMutation({
    mutationFn: async (values: CreateClientOnboardingReq) =>
      unwrapOnboarding(await createClientOnboarding(values)),
    onSuccess: (onboarding) => {
      setSubmittedId(onboarding.id);
      setStep(2);
    },
  });

  const form = useForm({
    defaultValues: {
      company_name: "",
      country_code: "US",
      registration_number: "",
      company_email: "",
      phone: "",
      address: "",
    } satisfies WizardValues,
    onSubmit: async ({ value }) => {
      await createMutation.mutateAsync(toRequest(value));
    },
  });

  const activeOnboarding = onboardingQuery.data;

  return (
    <section className="workspace-page">
      <header className="workspace-header">
        <div>
          <p className="eyebrow">Client onboarding</p>
          <h1>Register an originator business</h1>
        </div>
        <a className="text-link" href="/dashboard">
          Dashboard
        </a>
      </header>

      <div className="wizard-layout">
        <aside className="wizard-steps" aria-label="Onboarding steps">
          {steps.map((label, index) => (
            <button
              key={label}
              className="wizard-step"
              data-active={step === index ? "true" : undefined}
              type="button"
              onClick={() => setStep(index)}
            >
              <span>{index + 1}</span>
              {label}
            </button>
          ))}
        </aside>

        <form
          className="wizard-panel"
          onSubmit={(event) => {
            event.preventDefault();
            event.stopPropagation();
            void form.handleSubmit();
          }}
        >
          {step === 0 ? (
            <div className="wizard-fields">
              <PanelTitle icon={Building2} title="Company identity" />
              <form.Field
                name="company_name"
                validators={{
                  onChange: ({ value }) =>
                    value.trim().length === 0 ? "Company name is required" : undefined,
                }}
              >
                {(field) => (
                  <Field label="Company name" name={field.name} error={field.state.meta.errors}>
                    <Input
                      id={field.name}
                      value={field.state.value}
                      onBlur={field.handleBlur}
                      onChange={(event) => field.handleChange(event.target.value)}
                    />
                  </Field>
                )}
              </form.Field>
              <div className="field-grid">
                <form.Field
                  name="country_code"
                  validators={{
                    onChange: ({ value }) =>
                      value.trim().length === 2 ? undefined : "Use a 2-letter country code",
                  }}
                >
                  {(field) => (
                    <Field label="Country" name={field.name} error={field.state.meta.errors}>
                      <Input
                        id={field.name}
                        maxLength={2}
                        value={field.state.value}
                        onBlur={field.handleBlur}
                        onChange={(event) => field.handleChange(event.target.value.toUpperCase())}
                      />
                    </Field>
                  )}
                </form.Field>
                <form.Field name="registration_number">
                  {(field) => (
                    <Field label="Registration number" name={field.name}>
                      <Input
                        id={field.name}
                        value={field.state.value}
                        onBlur={field.handleBlur}
                        onChange={(event) => field.handleChange(event.target.value)}
                      />
                    </Field>
                  )}
                </form.Field>
              </div>
            </div>
          ) : null}

          {step === 1 ? (
            <div className="wizard-fields">
              <PanelTitle icon={Send} title="Contact details" />
              <form.Field
                name="company_email"
                validators={{
                  onChange: ({ value }) =>
                    value.length === 0 || value.includes("@") ? undefined : "Enter a valid email",
                }}
              >
                {(field) => (
                  <Field label="Company email" name={field.name} error={field.state.meta.errors}>
                    <Input
                      id={field.name}
                      type="email"
                      value={field.state.value}
                      onBlur={field.handleBlur}
                      onChange={(event) => field.handleChange(event.target.value)}
                    />
                  </Field>
                )}
              </form.Field>
              <form.Field name="phone">
                {(field) => (
                  <Field label="Phone" name={field.name}>
                    <Input
                      id={field.name}
                      value={field.state.value}
                      onBlur={field.handleBlur}
                      onChange={(event) => field.handleChange(event.target.value)}
                    />
                  </Field>
                )}
              </form.Field>
              <form.Field name="address">
                {(field) => (
                  <Field label="Business address" name={field.name}>
                    <Input
                      id={field.name}
                      value={field.state.value}
                      onBlur={field.handleBlur}
                      onChange={(event) => field.handleChange(event.target.value)}
                    />
                  </Field>
                )}
              </form.Field>
            </div>
          ) : null}

          {step === 2 ? (
            <div className="wizard-fields">
              <PanelTitle icon={CheckCircle2} title="Review and submit" />
              <dl className="review-list">
                {Object.entries(form.state.values).map(([key, value]) => (
                  <div key={key}>
                    <dt>{labelize(key)}</dt>
                    <dd>{value || "Not provided"}</dd>
                  </div>
                ))}
              </dl>
              {activeOnboarding ? <StatusPanel onboarding={activeOnboarding} /> : null}
              {createMutation.error ? (
                <p className="field-error">{createMutation.error.message}</p>
              ) : null}
            </div>
          ) : null}

          <footer className="wizard-actions">
            <Button
              disabled={step === 0}
              type="button"
              variant="outline"
              onClick={() => setStep((current) => Math.max(0, current - 1))}
            >
              <ArrowLeft aria-hidden="true" />
              Back
            </Button>
            {step < 2 ? (
              <Button type="button" onClick={() => setStep((current) => current + 1)}>
                Next
                <ArrowRight aria-hidden="true" />
              </Button>
            ) : (
              <Button disabled={createMutation.isPending || submittedId !== null} type="submit">
                {createMutation.isPending ? "Submitting" : "Submit"}
                <Send aria-hidden="true" />
              </Button>
            )}
          </footer>
        </form>
      </div>
    </section>
  );
}

function toRequest(values: WizardValues): CreateClientOnboardingReq {
  return {
    company_name: values.company_name.trim(),
    country_code: values.country_code.trim().toUpperCase(),
    company_email: optional(values.company_email),
    phone: optional(values.phone),
    registration_number: optional(values.registration_number),
    address: optional(values.address),
  };
}

function optional(value: string) {
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : undefined;
}

function unwrapOnboarding(response: Awaited<ReturnType<typeof createClientOnboarding>>) {
  const onboarding = response.data.data;
  if (!onboarding) {
    throw new Error("Onboarding response did not include data");
  }
  return onboarding;
}

function Field({
  children,
  error,
  label,
  name,
}: {
  children: React.ReactNode;
  error?: unknown[];
  label: string;
  name: string;
}) {
  return (
    <div className="form-field">
      <Label htmlFor={name}>{label}</Label>
      {children}
      <span className="field-error">{error?.map(String).join(", ")}</span>
    </div>
  );
}

function PanelTitle({
  icon: Icon,
  title,
}: {
  icon: React.ComponentType<React.SVGProps<SVGSVGElement>>;
  title: string;
}) {
  return (
    <div className="panel-title">
      <Icon aria-hidden="true" />
      <h2>{title}</h2>
    </div>
  );
}

function StatusPanel({ onboarding }: { onboarding: ClientOnboarding }) {
  const label =
    onboarding.status === "kyb_pending"
      ? "KYB in progress"
      : onboarding.status === "manual_review_pending"
        ? "Manual review pending"
        : labelize(onboarding.status);

  return (
    <div className="status-panel" data-status={onboarding.status}>
      <Clock3 aria-hidden="true" />
      <div>
        <strong>{label}</strong>
        <span>{onboarding.created_party_id ?? onboarding.temporal_workflow_id}</span>
      </div>
    </div>
  );
}

function labelize(value: string) {
  return value.replaceAll("_", " ").replace(/\b\w/g, (letter) => letter.toUpperCase());
}

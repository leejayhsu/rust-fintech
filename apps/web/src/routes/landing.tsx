import { useForm } from "@tanstack/react-form";
import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { ArrowRight, BadgeCheck, Landmark, LockKeyhole, ShieldCheck } from "lucide-react";

import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { signin, signup, type AuthCredentials } from "@/lib/auth-api";

const testCredentials = {
  email: "leejayhsu@gmail.com",
  password: "asdfasdf",
} satisfies AuthCredentials;

export function LandingRoute() {
  const navigate = useNavigate();

  const signinMutation = useMutation({
    mutationFn: signin,
    onSuccess: async () => {
      await navigate({ to: "/dashboard" });
    },
  });

  const signupMutation = useMutation({
    mutationFn: signup,
    onSuccess: () => {
      signinMutation.mutate(form.state.values);
    },
  });

  const form = useForm({
    defaultValues: testCredentials,
    onSubmit: async ({ value }) => {
      signinMutation.mutate(value);
    },
  });

  const authError = signinMutation.error?.message ?? signupMutation.error?.message;
  const isSubmitting = signinMutation.isPending || signupMutation.isPending;
  const canOfferSignup = signinMutation.isError && !signupMutation.isSuccess;

  return (
    <section className="landing-page">
      <div className="landing-hero">
        <header className="landing-nav">
          <div className="brand-mark">
            <Landmark aria-hidden="true" />
            <span>Rust Fintech</span>
          </div>
          <span className="api-status">
            <BadgeCheck aria-hidden="true" />
            API ready
          </span>
        </header>

        <div className="landing-copy">
          <p className="eyebrow">Ledger operations</p>
          <h1>Move money with a clearer control room.</h1>
          <p>
            Review balances, counterparties, and journal activity from one quiet workspace built
            around the Rust API.
          </p>
        </div>

        <div className="landing-metrics" aria-label="Operations snapshot">
          <div>
            <span>Available</span>
            <strong>$12.8k</strong>
          </div>
          <div>
            <span>Pending</span>
            <strong>3</strong>
          </div>
          <div>
            <span>Session</span>
            <strong>Cookie</strong>
          </div>
        </div>
      </div>

      <div className="login-pane">
        <Card className="login-card">
          <CardHeader>
            <div className="login-icon">
              <LockKeyhole aria-hidden="true" />
            </div>
            <CardTitle>Sign in</CardTitle>
            <CardDescription>Use your workspace credentials to open the dashboard.</CardDescription>
          </CardHeader>
          <CardContent>
            <form
              className="login-form"
              onSubmit={(event) => {
                event.preventDefault();
                event.stopPropagation();
                void form.handleSubmit();
              }}
            >
              <form.Field
                name="email"
                validators={{
                  onChange: ({ value }) =>
                    /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value)
                      ? undefined
                      : "Enter a valid email address",
                }}
              >
                {(field) => (
                  <div className="form-field">
                    <Label htmlFor={field.name}>Email</Label>
                    <Input
                      autoComplete="email"
                      id={field.name}
                      name={field.name}
                      type="email"
                      value={field.state.value}
                      onBlur={field.handleBlur}
                      onChange={(event) => field.handleChange(event.target.value)}
                    />
                    <FieldError errors={field.state.meta.errors} />
                  </div>
                )}
              </form.Field>

              <form.Field
                name="password"
                validators={{
                  onChange: ({ value }) =>
                    value.length >= 8 ? undefined : "Password must be at least 8 characters",
                }}
              >
                {(field) => (
                  <div className="form-field">
                    <Label htmlFor={field.name}>Password</Label>
                    <Input
                      autoComplete="current-password"
                      id={field.name}
                      name={field.name}
                      type="password"
                      value={field.state.value}
                      onBlur={field.handleBlur}
                      onChange={(event) => field.handleChange(event.target.value)}
                    />
                    <FieldError errors={field.state.meta.errors} />
                  </div>
                )}
              </form.Field>

              {authError ? (
                <Alert className="border-destructive/30 bg-destructive/5">
                  <AlertDescription>{authError}</AlertDescription>
                </Alert>
              ) : null}

              <Button className="w-full" disabled={isSubmitting} size="lg" type="submit">
                {signinMutation.isPending ? "Signing in" : "Sign in"}
                <ArrowRight aria-hidden="true" />
              </Button>

              {canOfferSignup ? (
                <Button
                  className="w-full"
                  disabled={signupMutation.isPending}
                  type="button"
                  variant="outline"
                  onClick={() => signupMutation.mutate(form.state.values)}
                >
                  {signupMutation.isPending ? "Creating account" : "Create this test account"}
                </Button>
              ) : null}
            </form>
          </CardContent>
        </Card>

        <div className="security-note">
          <ShieldCheck aria-hidden="true" />
          <span>Authenticated with the API session cookie.</span>
        </div>
      </div>
    </section>
  );
}

function FieldError({ errors }: { errors: unknown[] }) {
  const message = errors.map(String).join(", ");

  return <p className="field-error">{message}</p>;
}

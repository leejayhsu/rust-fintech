import { useForm } from "@tanstack/react-form";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import type { ComponentType, SVGProps } from "react";
import {
  ArrowUpRight,
  Bell,
  CreditCard,
  Gauge,
  Landmark,
  LogOut,
  ReceiptText,
  Search,
  Settings,
  ShieldCheck,
  Users,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { getCurrentUser } from "@/lib/auth-api";

type TransferFormValues = {
  recipient: string;
  amount: number;
};

type SidebarItem = {
  label: string;
  icon: ComponentType<SVGProps<SVGSVGElement>>;
  active?: boolean;
};

const sidebarItems = [
  { label: "Overview", icon: Gauge, active: true },
  { label: "Transfers", icon: ArrowUpRight },
  { label: "Accounts", icon: CreditCard },
  { label: "Counterparties", icon: Users },
  { label: "Statements", icon: ReceiptText },
] satisfies SidebarItem[];

const accountSummaryQueryKey = ["account-summary"] as const;

async function getAccountSummary() {
  return {
    balance: 12840.52,
    currency: "USD",
    pendingTransfers: 3,
  };
}

async function createTransfer(values: TransferFormValues) {
  return values;
}

export function DashboardRoute() {
  const navigate = useNavigate();
  const accountSummary = useQuery({
    queryKey: accountSummaryQueryKey,
    queryFn: getAccountSummary,
  });

  const currentUser = useQuery({
    queryKey: ["current-user"],
    queryFn: getCurrentUser,
  });

  const transferMutation = useMutation({
    mutationFn: createTransfer,
  });

  const form = useForm({
    defaultValues: {
      recipient: "",
      amount: 0,
    } satisfies TransferFormValues,
    onSubmit: async ({ value }) => {
      await transferMutation.mutateAsync(value);
    },
  });

  return (
    <section className="dashboard-shell">
      <AppSidebar isAdmin={currentUser.data?.role === "admin"} />

      <div className="dashboard">
        <header className="dashboard-header">
          <div>
            <p className="eyebrow">Rust Fintech</p>
            <h1>Operations Console</h1>
          </div>
          <div className="dashboard-actions">
            <Button aria-label="Search" size="icon" type="button" variant="outline">
              <Search aria-hidden="true" />
            </Button>
            <Button aria-label="Notifications" size="icon" type="button" variant="outline">
              <Bell aria-hidden="true" />
            </Button>
            <div className="status-pill">Signed in</div>
          </div>
        </header>

        <div className="dashboard-grid">
          <article className="metric-panel">
            <span>Available Balance</span>
            <strong>
              {accountSummary.data
                ? new Intl.NumberFormat("en-US", {
                    style: "currency",
                    currency: accountSummary.data.currency,
                  }).format(accountSummary.data.balance)
                : "Loading"}
            </strong>
          </article>

          <article className="metric-panel">
            <span>Pending Transfers</span>
            <strong>{accountSummary.data?.pendingTransfers ?? "Loading"}</strong>
          </article>

          <article className="metric-panel action-panel">
            <span>Client setup</span>
            <Button type="button" onClick={() => void navigate({ to: "/onboarding/client" })}>
              Onboard Client
            </Button>
          </article>
        </div>

        <form
          className="transfer-form"
          onSubmit={(event) => {
            event.preventDefault();
            event.stopPropagation();
            void form.handleSubmit();
          }}
        >
          <h2>New Transfer</h2>

          <form.Field
            name="recipient"
            validators={{
              onChange: ({ value }) =>
                value.trim().length === 0 ? "Recipient is required" : undefined,
            }}
          >
            {(field) => (
              <div className="form-field">
                <Label htmlFor={field.name}>Recipient</Label>
                <Input
                  id={field.name}
                  name={field.name}
                  value={field.state.value}
                  onBlur={field.handleBlur}
                  onChange={(event) => field.handleChange(event.target.value)}
                />
                <span className="field-error">{field.state.meta.errors.join(", ")}</span>
              </div>
            )}
          </form.Field>

          <form.Field
            name="amount"
            validators={{
              onChange: ({ value }) =>
                value <= 0 ? "Amount must be greater than zero" : undefined,
            }}
          >
            {(field) => (
              <div className="form-field">
                <Label htmlFor={field.name}>Amount</Label>
                <Input
                  id={field.name}
                  min="0"
                  name={field.name}
                  step="0.01"
                  type="number"
                  value={field.state.value}
                  onBlur={field.handleBlur}
                  onChange={(event) => field.handleChange(event.target.valueAsNumber)}
                />
                <span className="field-error">{field.state.meta.errors.join(", ")}</span>
              </div>
            )}
          </form.Field>

          <Button type="submit" disabled={transferMutation.isPending}>
            {transferMutation.isPending ? "Submitting" : "Queue Transfer"}
          </Button>
        </form>
      </div>
    </section>
  );
}

function AppSidebar({ isAdmin }: { isAdmin: boolean }) {
  const navigate = useNavigate();

  return (
    <aside className="app-sidebar" aria-label="Primary navigation">
      <div className="sidebar-brand">
        <div className="sidebar-brand-icon">
          <Landmark aria-hidden="true" />
        </div>
        <div>
          <strong>Rust Fintech</strong>
          <span>Workspace</span>
        </div>
      </div>

      <nav className="sidebar-nav">
        {sidebarItems.map((item) => (
          <Button
            key={item.label}
            className="sidebar-nav-item"
            data-active={item.active ? "true" : undefined}
            type="button"
            variant="ghost"
          >
            <item.icon aria-hidden="true" />
            <span>{item.label}</span>
          </Button>
        ))}
        {isAdmin ? (
          <Button
            className="sidebar-nav-item"
            type="button"
            variant="ghost"
            onClick={() => void navigate({ to: "/admin/onboarding" })}
          >
            <ShieldCheck aria-hidden="true" />
            <span>Admin Review</span>
          </Button>
        ) : null}
      </nav>

      <div className="sidebar-footer">
        <Button className="sidebar-nav-item" type="button" variant="ghost">
          <Settings aria-hidden="true" />
          <span>Settings</span>
        </Button>
        <Button className="sidebar-nav-item" type="button" variant="ghost">
          <LogOut aria-hidden="true" />
          <span>Sign out</span>
        </Button>
      </div>
    </aside>
  );
}

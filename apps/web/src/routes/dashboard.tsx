import { useForm } from "@tanstack/react-form";
import { useMutation, useQuery } from "@tanstack/react-query";

type TransferFormValues = {
  recipient: string;
  amount: number;
};

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
  const accountSummary = useQuery({
    queryKey: accountSummaryQueryKey,
    queryFn: getAccountSummary,
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
    <section className="dashboard">
      <header className="dashboard-header">
        <div>
          <p className="eyebrow">Rust Fintech</p>
          <h1>Operations Console</h1>
        </div>
        <div className="status-pill">API ready</div>
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
            <label>
              Recipient
              <input
                name={field.name}
                value={field.state.value}
                onBlur={field.handleBlur}
                onChange={(event) => field.handleChange(event.target.value)}
              />
              <span>{field.state.meta.errors.join(", ")}</span>
            </label>
          )}
        </form.Field>

        <form.Field
          name="amount"
          validators={{
            onChange: ({ value }) => (value <= 0 ? "Amount must be greater than zero" : undefined),
          }}
        >
          {(field) => (
            <label>
              Amount
              <input
                min="0"
                name={field.name}
                step="0.01"
                type="number"
                value={field.state.value}
                onBlur={field.handleBlur}
                onChange={(event) => field.handleChange(event.target.valueAsNumber)}
              />
              <span>{field.state.meta.errors.join(", ")}</span>
            </label>
          )}
        </form.Field>

        <button type="submit" disabled={transferMutation.isPending}>
          {transferMutation.isPending ? "Submitting" : "Queue Transfer"}
        </button>
      </form>
    </section>
  );
}

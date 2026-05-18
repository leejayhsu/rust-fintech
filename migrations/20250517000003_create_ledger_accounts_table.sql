CREATE TABLE ledger_accounts (
    id VARCHAR(32) PRIMARY KEY,
    pending_balance NUMERIC(28,4) NOT NULL DEFAULT 0,
    available_balance NUMERIC(28,4) NOT NULL DEFAULT 0,
    posted_balance NUMERIC(28,4) NOT NULL DEFAULT 0,
    is_neg_balance_allowed BOOLEAN NOT NULL DEFAULT FALSE,
    currency_code VARCHAR(3) NOT NULL REFERENCES currencies(code),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ledger_accounts_currency ON ledger_accounts(currency_code);

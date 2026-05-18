CREATE TABLE ledger_accounts (
    id VARCHAR(32) PRIMARY KEY,
    owner_id VARCHAR(32) NOT NULL REFERENCES users(id),
    name VARCHAR(255),
    is_neg_balance_allowed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ledger_accounts_owner ON ledger_accounts(owner_id);

CREATE TABLE ledger_account_balances (
    account_id VARCHAR(32) NOT NULL REFERENCES ledger_accounts(id) ON DELETE CASCADE,
    currency_code VARCHAR(3) NOT NULL REFERENCES currencies(code),
    pending_balance NUMERIC(28,4) NOT NULL DEFAULT 0,
    available_balance NUMERIC(28,4) NOT NULL DEFAULT 0,
    posted_balance NUMERIC(28,4) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (account_id, currency_code)
);

CREATE INDEX idx_ledger_account_balances_currency ON ledger_account_balances(currency_code);

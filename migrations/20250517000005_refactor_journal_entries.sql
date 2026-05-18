DROP TABLE IF EXISTS ledger_entries;

CREATE TABLE journal_entries (
    id VARCHAR(32) PRIMARY KEY,
    status VARCHAR(7) NOT NULL CHECK (status IN ('pending', 'posted')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_journal_entries_status ON journal_entries(status);

CREATE TABLE ledger_entries (
    id VARCHAR(32) PRIMARY KEY,
    journal_entry_id VARCHAR(32) NOT NULL REFERENCES journal_entries(id),
    account_id VARCHAR(32) NOT NULL REFERENCES ledger_accounts(id),
    currency_code VARCHAR(3) NOT NULL REFERENCES currencies(code),
    amount NUMERIC(28,4) NOT NULL,
    direction VARCHAR(6) NOT NULL CHECK (direction IN ('debit', 'credit')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ledger_entries_journal ON ledger_entries(journal_entry_id);
CREATE INDEX idx_ledger_entries_account ON ledger_entries(account_id);
CREATE INDEX idx_ledger_entries_created_at ON ledger_entries(created_at);

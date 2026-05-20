CREATE TABLE parties (
    id VARCHAR(32) PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    country_code VARCHAR(2),
    type VARCHAR(20) NOT NULL CHECK (type IN ('individual', 'business')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_parties_name ON parties(name);
CREATE INDEX idx_parties_email ON parties(email);
CREATE INDEX idx_parties_type ON parties(type);

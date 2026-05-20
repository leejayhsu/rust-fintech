CREATE TABLE party_members (
    id TEXT PRIMARY KEY,
    party_id TEXT NOT NULL REFERENCES parties(id) ON DELETE CASCADE,
    parent_party_member_id TEXT,
    legal_name TEXT NOT NULL,
    type TEXT NOT NULL CHECK (type IN ('individual', 'business')),
    address TEXT,
    title TEXT,
    is_legal_rep BOOLEAN NOT NULL DEFAULT FALSE,
    is_ubo BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT party_members_not_own_parent
        CHECK (parent_party_member_id IS NULL OR parent_party_member_id <> id),

    CONSTRAINT party_members_id_party_unique
        UNIQUE (id, party_id),

    CONSTRAINT party_members_parent_same_party
        FOREIGN KEY (parent_party_member_id, party_id)
        REFERENCES party_members(id, party_id)
        ON DELETE CASCADE
);

CREATE INDEX idx_party_members_party_id ON party_members(party_id);
CREATE INDEX idx_party_members_parent ON party_members(party_id, parent_party_member_id);
CREATE INDEX idx_party_members_type ON party_members(type);
CREATE INDEX idx_party_members_legal_rep ON party_members(party_id) WHERE is_legal_rep = TRUE;
CREATE INDEX idx_party_members_ubo ON party_members(party_id) WHERE is_ubo = TRUE;

CREATE TABLE gov_ids (
    id TEXT PRIMARY KEY,
    party_id TEXT REFERENCES parties(id) ON DELETE CASCADE,
    party_member_id TEXT REFERENCES party_members(id) ON DELETE CASCADE,
    type TEXT NOT NULL,
    value TEXT NOT NULL,
    issuing_country_code TEXT,
    issued_at DATE,
    expires_at DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT gov_ids_exactly_one_owner
        CHECK (num_nonnulls(party_id, party_member_id) = 1)
);

CREATE INDEX idx_gov_ids_party_id ON gov_ids(party_id);
CREATE INDEX idx_gov_ids_party_member_id ON gov_ids(party_member_id);
CREATE INDEX idx_gov_ids_type ON gov_ids(type);
CREATE UNIQUE INDEX idx_gov_ids_unique_party_type ON gov_ids(party_id, type) WHERE party_id IS NOT NULL;
CREATE UNIQUE INDEX idx_gov_ids_unique_party_member_type ON gov_ids(party_member_id, type) WHERE party_member_id IS NOT NULL;

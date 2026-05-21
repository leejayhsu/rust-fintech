CREATE TABLE client_onboardings (
    id TEXT PRIMARY KEY,
    submitted_by_user_id TEXT NOT NULL REFERENCES users(id),
    company_name TEXT NOT NULL,
    company_email TEXT,
    phone TEXT,
    country_code TEXT NOT NULL,
    registration_number TEXT,
    address TEXT,
    status TEXT NOT NULL CHECK (
        status IN (
            'draft',
            'kyb_pending',
            'manual_review_pending',
            'approved',
            'rejected',
            'failed'
        )
    ),
    temporal_workflow_id TEXT UNIQUE,
    kyb_vendor_a_status TEXT,
    kyb_vendor_a_response JSONB,
    kyb_vendor_b_status TEXT,
    kyb_vendor_b_response JSONB,
    reviewed_by_user_id TEXT REFERENCES users(id),
    reviewed_at TIMESTAMPTZ,
    review_note TEXT,
    created_party_id TEXT REFERENCES parties(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_client_onboardings_submitted_by ON client_onboardings(submitted_by_user_id);
CREATE INDEX idx_client_onboardings_status ON client_onboardings(status);
CREATE INDEX idx_client_onboardings_workflow_id ON client_onboardings(temporal_workflow_id);

ALTER TABLE users
ADD COLUMN role TEXT NOT NULL DEFAULT 'user'
CHECK (role IN ('user', 'admin'));

UPDATE users
SET role = 'admin'
WHERE email = 'leejayhsu@gmail.com';

ALTER TABLE parties
ADD COLUMN role TEXT NOT NULL DEFAULT 'counterparty'
CHECK (role IN ('originator', 'beneficiary', 'counterparty'));

CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_parties_role ON parties(role);

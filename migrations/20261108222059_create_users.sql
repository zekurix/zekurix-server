-- Create users table
CREATE TABLE users (
    id UUID PRIMARY KEY
);

--- Create identities table
CREATE TABLE identities (
    issuer VARCHAR(1024) NOT NULL CHECK (length(issuer) > 0),
    subject VARCHAR(1024) NOT NULL CHECK (length(subject) > 0),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (issuer, subject)
);

-- Create users table
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    CONSTRAINT users_username_format CHECK (username ~ '^[A-Za-z0-9_-]{3,64}$')
);

--- Create identities table
CREATE TABLE identities (
    issuer VARCHAR(1024) NOT NULL CHECK (length(issuer) > 0),
    subject VARCHAR(1024) NOT NULL CHECK (length(subject) > 0),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (issuer, subject)
);

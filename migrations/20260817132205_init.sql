-- Create users table
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    CONSTRAINT users_username_format CHECK (username ~ '^[A-Za-z0-9_-]{3,64}$')
);

--- Create identities table
CREATE TABLE identities (
    issuer TEXT NOT NULL CHECK (issuer <> ''),
    subject TEXT NOT NULL CHECK (subject <> ''),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (issuer, subject)
);

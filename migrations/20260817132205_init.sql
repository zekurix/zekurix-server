-- Create users table
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    CONSTRAINT users_username_format CHECK (username ~ '^[A-Za-z0-9_-]{3,64}$')
);

-- Create permissions table
CREATE TABLE permissions (
    name TEXT PRIMARY KEY CHECK (length(name) > 0)
);

-- Insert the permissions
INSERT INTO permissions (name) VALUES
    ('roles:create'),
    ('roles:read'),
    ('roles:update'),
    ('roles:delete'),
    ('roles:assign'),
    ('roles:unassign');

-- Create roles table
CREATE TABLE roles (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE CHECK (length(name) > 0)
);

-- Create roles - permissions association table
CREATE TABLE role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission TEXT NOT NULL REFERENCES permissions(name) ON DELETE RESTRICT,
    PRIMARY KEY (role_id, permission)
);

-- Create users - roles association table
CREATE TABLE user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id));

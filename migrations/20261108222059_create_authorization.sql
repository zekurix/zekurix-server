-- Create permissions table
CREATE TABLE permissions (
    name VARCHAR(255) PRIMARY KEY CHECK (length(name) > 0)
);

-- Insert the permissions
INSERT INTO permissions (name) VALUES
    ('permissions:read'),
    ('roles:create'),
    ('roles:read'),
    ('roles:update'),
    ('roles:delete'),
    ('roles:assign'),
    ('roles:unassign');

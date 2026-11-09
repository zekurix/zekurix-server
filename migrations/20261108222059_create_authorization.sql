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

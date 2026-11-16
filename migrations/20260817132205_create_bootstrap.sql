-- Create completed bootstraps table
CREATE TABLE completed_bootstraps (
    name TEXT PRIMARY KEY CHECK (length(name) > 0)
);

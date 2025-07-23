CREATE TABLE buckets (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    path VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE commits (
    id UUID PRIMARY KEY,
    bucket_id UUID NOT NULL,
    message TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (bucket_id) REFERENCES buckets (id)
);

CREATE TABLE files (
    id UUID PRIMARY KEY,
    commit_id UUID NOT NULL,
    file_path VARCHAR(255) NOT NULL,
    hash VARCHAR(64) NOT NULL,
    FOREIGN KEY (commit_id) REFERENCES commits (id),
    UNIQUE (commit_id, file_path, hash)
); 
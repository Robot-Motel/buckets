
# Database Schema

## Tables and Relationships

### 1. `buckets`
This table stores information about buckets.

- **SQL**:
  ```sql
  CREATE TABLE buckets ( 
      id UUID PRIMARY KEY,
      name TEXT NOT NULL,
      path TEXT NOT NULL
  );
  ```

- **Columns**:

| Column | Type  | Constraints      |
|--------|-------|------------------|
| `id`   | UUID  | PRIMARY KEY      |
| `name` | TEXT  | NOT NULL         |
| `path` | TEXT  | NOT NULL         |

---

### 2. `commits`
This table records commits related to specific buckets.

- **SQL**:
  ```sql
  CREATE TABLE commits (
      id UUID PRIMARY KEY,
      bucket_id UUID NOT NULL,
      message TEXT NOT NULL,
      created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
      FOREIGN KEY (bucket_id) REFERENCES buckets (id)
  );
  ```

- **Columns**:

| Column       | Type      | Constraints                                   |
|--------------|-----------|-----------------------------------------------|
| `id`         | UUID      | PRIMARY KEY                                  |
| `bucket_id`  | UUID      | NOT NULL, FOREIGN KEY → `buckets(id)`         |
| `message`    | TEXT      | NOT NULL                                     |
| `created_at` | TIMESTAMP | NOT NULL, DEFAULT CURRENT_TIMESTAMP          |

- **Relationships**:
    - Each commit is associated with a specific bucket via the `bucket_id` foreign key.

---

### 3. `files`
This table tracks the files associated with each commit.

- **SQL**:
  ```sql
  CREATE TABLE files (
      id UUID PRIMARY KEY,
      commit_id UUID NOT NULL,
      file_path TEXT NOT NULL,
      hash TEXT NOT NULL,
      FOREIGN KEY (commit_id) REFERENCES commits (id),
      UNIQUE (commit_id, file_path, hash)
  );
  ```

- **Columns**:

| Column       | Type  | Constraints                                   |
|--------------|-------|-----------------------------------------------|
| `id`         | UUID  | PRIMARY KEY                                  |
| `commit_id`  | UUID  | NOT NULL, FOREIGN KEY → `commits(id)`         |
| `file_path`  | TEXT  | NOT NULL                                     |
| `hash`       | TEXT  | NOT NULL                                     |
| *(Unique)*   |       | UNIQUE(commit_id, file_path, hash)           |

- **Relationships**:
    - Each file is associated with a specific commit via the `commit_id` foreign key.

---

## Relationships Summary

1. `buckets` → `commits`: A bucket can have multiple commits. (`buckets.id = commits.bucket_id`)
2. `commits` → `files`: A commit can reference multiple files. (`commits.id = files.commit_id`)

# Status command
The status command is used to check the status of the current repository or if 
executed within a bucket the status of the bucket.

## Status of the repository
The status of the repository is displayed in the following format:
```
Repository: <repository_name>
Buckets: <number_of_buckets>
List of buckets:
    <bucket_name_1> <local_path_1>
    <bucket_name_2> <local_path_2>
    ...
    <bucket_name_n> <local_path_n>
```

## Status of the bucket
For every file in the bucket (including files in subdirectories) the status of 
the file is displayed in the following format:
```
unknown:   <file_name>
new file:  <file_name>
committed: <file_name>
modified:  <file_name>
deleted:   <file_name>
```
This is based on the`CommitStatus` enum 
```rust
pub enum CommitStatus {
    Unknown,
    New,
    Committed,
    Modified,
    Deleted,
}
```
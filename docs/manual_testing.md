# Manual Tests

## Preparation

On Windows you can create an alias for the `bucket.exe` command

```bash
cd buckets # Go into the root of the bucket repository
cargo install --path . # Install buckets
Get-Command buckets.exe # Gives the location
Set-Alias buckets "C:\Users\WindowsUser\.cargo\bin\buckets.exe" # Create alias
winget install DuckDB.cli # Install command line DuckDB
```

## Initialize a repository

### Steps

```bash
bucket init test_repo
```

### Expected results

Directory structure and files

```bash
./test_repo/
./test_repo/.buckets/
./test_repo/.buckets/buckets.db
./test_repo/.buckets/config
```

`buckets.db` is a DuckDB database

```bash
duckdb ./test_repo/.buckets/buckets.db
D show tables;
┌─────────┐
│  name   │
│ varchar │
├─────────┤
│ buckets │
│ commits │
│ files   │
└─────────┘
D describe buckets;
┌─────────────┬─────────────┬─────────┬─────────┬─────────┬─────────┐
│ column_name │ column_type │  null   │   key   │ default │  extra  │
│   varchar   │   varchar   │ varchar │ varchar │ varchar │ varchar │
├─────────────┼─────────────┼─────────┼─────────┼─────────┼─────────┤
│ id          │ UUID        │ NO      │ PRI     │         │         │
│ name        │ VARCHAR     │ NO      │         │         │         │
│ path        │ VARCHAR     │ NO      │         │         │         │
└─────────────┴─────────────┴─────────┴─────────┴─────────┴─────────┘
D select * from buckets;
┌──────┬─────────┬─────────┐
│  id  │  name   │  path   │
│ uuid │ varchar │ varchar │
├──────┴─────────┴─────────┤
│          0 rows          │
└──────────────────────────┘
```

## Create bucket

### Steps

```bash
buckets create test_bucket
```

### Expected results

UUID will be different

```bash
duckdb ./test_repo/.buckets/buckets.db
D select * from buckets
┌──────────────────────────────────────┬─────────────┬─────────────┐
│                  id                  │    name     │    path     │
│                 uuid                 │   varchar   │   varchar   │
├──────────────────────────────────────┼─────────────┼─────────────┤
│ c1fdcb0b-757e-4631-bbb6-272c98b49424 │ test_bucket │ test_bucket │
└──────────────────────────────────────┴─────────────┴─────────────┘
```

## Commit file

### Steps

```bash
cd test_bucket
New-Item boat.blend -ItemType File
"This is a blend file" | Out-File -FilePath .\boat.blend
buckets commit 
```

### Expected results

#### Files and directories

```bash
./test_repo/test_bucket/
./test_repo/test_bucket/.b/
./test_repo/test_bucket/.b/info
./test_repo/test_bucket/.b/storage/
./test_repo/test_bucket/.b/storage/1496dd00f4648d8c368...
```

#### Database

```bash
duckdb .\test_repo\.buckets\buckets.db
select * from commits;
┌──────────────────────────────────────┬──────────────────────────────────────┬─────────┬─────────────────────────┐
│                  id                  │              bucket_id               │ message │       created_at        │
│                 uuid                 │                 uuid                 │ varchar │        timestamp        │
├──────────────────────────────────────┼──────────────────────────────────────┼─────────┼─────────────────────────┤
│ 2d7a0558-f206-4659-9629-3bec710f984f │ c1fdcb0b-757e-4631-bbb6-272c98b49424 │         │ 2024-09-27 14:54:52.097 │
└──────────────────────────────────────┴──────────────────────────────────────┴─────────┴─────────────────────────┘
select * from files;
┌──────────────────────┬─────────────────────────────────────┬────────────┬──────────────────────────────────────────────────────────────────┐
│          id          │              commit_id              │ file_path  │                               hash                               │
│         uuid         │                uuid                 │  varchar   │                             varchar                              │
├──────────────────────┼─────────────────────────────────────┼────────────┼──────────────────────────────────────────────────────────────────┤
│ c687a296-272f-46c9.  │ 2d7a0558-f206-4659-9629-3bec710f9.  │ boat.blend │ 1496dd00f4648d8c36876585488eb09efc7428c499223e96664e520fb27fc9e3 │
└──────────────────────┴─────────────────────────────────────┴────────────┴──────────────────────────────────────────────────────────────────┘
```
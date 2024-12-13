# Rollback Command

The `rollback` command is used to discard all changes, or changes to a single file, in the bucket and restore the last commit. 

## Usage
Discard all changes in the bucket and restore the last commit:

```shell
bucket rollback
```

Discard changes to a specific file and restore the file as it was in the last commit:
```shell
bucket rollback [file] 
```

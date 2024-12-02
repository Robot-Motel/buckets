# Directory Layout
## Top level**

```bash
.\test_repo\
.\test_repo\.buckets\
.\test_repo\.buckets\buckets.db
.\test_repo\.buckets\config
```

`.buckets` Resides at the top level of the repository. Contains general information. 

`.buckets\config` Bucket repository configuration file.

`.buckets\buckets.db` Repository metadata storage. See [Database
Layout](database_layout.md)

## **Per bucket container**
Every bucket has the following layout:
```shell
.\test_repo\test_bucket\
.\test_repo\test_bucket\.b\
.\test_repo\test_bucket\.b\info
.\test_repo\test_bucket\.b\storage\
```
`.b` At the top of the bucket, contains bucket info and storage

`.b\config` Bucket configuration file. See [Bucket Configuration](repository_configuration)

`.b\storage\` Object storage for commited assets. See [Bucket object storage](object_storage_and_hashing)

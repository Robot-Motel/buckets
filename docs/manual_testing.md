# Manual Test Plan for Buckets CLI

## Test Environment Setup

### Prerequisites
- Rust toolchain installed
- PostgreSQL client tools (for PostgreSQL tests)

### Installation Methods

#### Option 1: Direct Installation
```bash
cd buckets
cargo install --path .
buckets --version
```

#### Option 2: Development Build
```bash
cd buckets
cargo build --release
./target/release/buckets --version
```

#### Option 3: Windows-specific Setup
```powershell
cd buckets
cargo install --path .
Get-Command buckets.exe
Set-Alias buckets "C:\Users\WindowsUser\.cargo\bin\buckets.exe"
buckets --version
```

## Test Suite

### TC001: Repository Initialization - Default (Embedded PostgreSQL)

**Objective:** Verify repository initialization with default embedded PostgreSQL backend

**Preconditions:** Clean test environment

**Test Steps:**
```bash
buckets init test_repo_postgres
```

**Expected Results:**
- Exit code: 0
- Console output: "Bucket repository initialized successfully."
- Directory structure:
  ```
  ./test_repo_postgres/
  ./test_repo_postgres/.buckets/
  ./test_repo_postgres/.buckets/config
  ./test_repo_postgres/.buckets/database_type
  ./test_repo_postgres/.buckets/postgres_data/
  ```
- Database type file contains: `postgresql`
- PostgreSQL data directory created

**Post-conditions:** Repository ready for bucket creation

---

### TC002: Repository Initialization - External Database

**Objective:** Verify repository initialization with an external database

**Preconditions:** 
- Clean test environment

**Test Steps:**
```bash
buckets init test_repo_external --external-database "postgres://user:password@localhost/db"
```

**Expected Results:**
- Exit code: 0
- Console output: "Bucket repository initialized successfully."
- Directory structure:
  ```
  ./test_repo_external/
  ./test_repo_external/.buckets/
  ./test_repo_external/.buckets/config
  ./test_repo_external/.buckets/database_type
  ```
- Config file contains the external database connection string

**Post-conditions:** Repository ready for bucket creation

---

### TC003: Database Option Validation

**Objective:** Verify proper validation of database type parameter

**Test Cases:**

#### TC003a: Valid Database Types
```bash
buckets init test_valid_postgres --database postgres  # Should succeed  
buckets init test_valid_postgresql --database postgresql # Should succeed
```

#### TC003b: Invalid Database Type
```bash
buckets init test_invalid --database mysql
```
**Expected Results:**
- Exit code: non-zero
- Error message: "Invalid database type 'mysql'. Valid options are: postgresql"

---

### TC004: Bucket Creation

**Objective:** Verify bucket creation functionality

**Preconditions:** Valid repository initialized (from TC001 or TC002)

**Test Steps:**
```bash
cd test_repo_postgres  # or test_repo_external
buckets create test_bucket
```

**Expected Results:**
- Exit code: 0
- Console output indicating successful bucket creation

**Post-conditions:** Bucket ready for file operations

---

### TC005: File Commit Operations

**Objective:** Verify file commit functionality across database backends

**Preconditions:** Bucket created (from TC004)

**Test Steps:**
```bash
cd test_bucket
echo "This is a test file" > test_file.txt
buckets commit "Add test file"
```

**Expected Results:**
- Exit code: 0
- File storage directory created: `./.b/storage/`
- Database records updated in commits and files tables
- File hash correctly stored

**Database Verification:**
```bash
# For PostgreSQL  
# Connect using appropriate PostgreSQL client with embedded server URL
```

---

### TC006: Cross-Platform Compatibility

**Objective:** Verify functionality across different operating systems

#### TC006a: Unix/Linux Commands
```bash
cd test_bucket
touch boat.blend
echo "Blender file content" > boat.blend
buckets commit "new boat"
```

#### TC006b: Windows Commands  
```powershell
cd test_bucket
New-Item boat.blend -ItemType File
"Blender file content" | Out-File -FilePath .\boat.blend
buckets commit "new boat"
```

**Expected Results:** Consistent behavior across platforms

---

### TC007: Status and File Tracking

**Objective:** Verify status reporting functionality

**Test Steps:**
```bash
cd test_bucket
echo "New file" > anchor.blend
buckets commit "new anchor"
echo "Modified content" > anchor.blend
touch rudder.blend
buckets status
```

**Expected Results:**
```
committed:    [previously committed files]
modified:     anchor.blend
new:          rudder.blend
```

---

### TC008: Rollback Functionality

**Objective:** Verify rollback operations

**Test Steps:**
```bash
buckets rollback
buckets status
```

**Expected Results:**
- Modified files restored to committed state
- Status shows clean working directory for committed files
- New files remain untracked

---

### TC009: Help and Documentation

**Objective:** Verify help system functionality

**Test Cases:**
```bash
buckets --help                           # General help
buckets init --help                      # Init command help  
buckets create --help                    # Create command help
buckets commit --help                    # Commit command help
```

**Expected Results:** 
- Comprehensive help text displayed
- Database option documented for init command
- All required parameters clearly indicated

---

### TC010: Error Handling and Edge Cases

**Objective:** Verify robust error handling

#### TC010a: Duplicate Repository
```bash
buckets init existing_repo
buckets init existing_repo  # Should fail
```

#### TC010b: Operations Outside Repository
```bash
mkdir /tmp/not_a_repo
cd /tmp/not_a_repo  
buckets create test_bucket  # Should fail
```

#### TC010c: Invalid Bucket Names
```bash
buckets create ""           # Empty name
buckets create "invalid/name"  # Invalid characters
```

**Expected Results:** Appropriate error messages and non-zero exit codes

---

## Test Execution Guidelines

### Pre-Test Setup
1. Clean environment with no existing test repositories
2. Verify Rust toolchain and dependencies installed
3. Build application with and without postgres feature
4. Prepare test data files as needed

### Test Data Management  
- Use consistent test file names and content
- Verify file hashes match expected values
- Clean up test repositories between test runs

### Pass/Fail Criteria
- **PASS:** All expected results achieved, exit codes correct
- **FAIL:** Any expected result not achieved, incorrect exit codes
- **BLOCKED:** Cannot execute due to environment/dependency issues

### Reporting
Document for each test case:
- Test case ID and description
- Execution timestamp
- Pass/Fail status
- Actual results vs expected results  
- Screenshots/logs for failures
- Environment details (OS, Rust version, etc.)


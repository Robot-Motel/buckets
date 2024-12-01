# Program Architecture Overview

Buckets is a modular CLI application, structured around commands and utilities, and follows the command-line tool pattern.


## Core Design
Buckets is a command-line application that uses subcommands to perform various operations, such as initializing repositories, creating buckets, committing changes, and checking statuses. Much like a typical for CLI tools inspired by Git-like interfaces.

## Key Architectural Components

### a. Command Handlers
- Each subcommand (e.g., init, create, commit, status) has a dedicated module, following the **Single Responsibility Principle**.
- Commands are implemented in separate modules under `commands/`, and each has an `execute()` function to handle its logic.
- Commands are defined in `args.rs` as part of a Command enum and mapped to their respective modules in `main.rs`.

### b. Argument Parsing
- **`args.rs`**:
    - Decouples argument parsing from execution logic. 
    - Uses `clap` for parsing arguments.
    - Defines subcommands (e.g. `InitCommand`, `CreateCommand`) with associated options (e.g., `--verbose`).
    

### c. Error Handling
- **`errors.rs`**:
    - Centralized error management with the `BucketError` enum.
    - Provides error messages via the `message` method.
    - Implements `From<io::Error>` for seamless error propagation with `?`.

### d. Utilities
- **`utils/`**:
    - Encapsulates reusable functions (e.g., directory validation in `checks.rs`).
    - Includes utility methods for finding directories, validating repositories, and managing bucket-specific logic.

### e. State Management
- **Thread-Local State**:
    - `CURRENT_DIR`: Stores the current working directory as a thread-local variable, ensuring consistent access across different threads.
    - `EXIT`: Tracks the program’s exit code for clean and centralized program termination.

### f. Entry Point
- **`main.rs`**:
    - Sets up the CLI using `clap`.
    - Delegates execution to the appropriate command handler using `dispatch`.
    - Handles error reporting and exit code management.

## Execution Flow

1. **Argument Parsing**:
    - CLI arguments are parsed using `clap` to identify the requested subcommand.

2. **Command Dispatch**:
    - `dispatch` function routes the request to the appropriate command handler (e.g., `commands::init::execute`).

3. **Command Execution**:
    - Each command handler performs its tasks (e.g., `init` initializes a repository).
    - Utility functions support operations like validating directories.

4. **Error Handling**:
    - Errors are propagated back to main.rs using the `Result` type.
    - Errors are logged, and the exit code is set to failure if needed.

5. **Program Termination**:
    - The exit code (`SUCCESS` or `FAILURE`) is managed via the thread-local `EXIT` variable and returned as the final program status.

## Module Structure

### Root Modules
| Module       | Responsibility                                   |
|--------------|-------------------------------------------------|
| `main.rs`    | Program entry point; dispatches commands.       |
| `args.rs`    | Defines CLI structure and subcommand options.   |
| `errors.rs`  | Centralized error types and messages.           |

### Commands
| Command   | File                  | Responsibility                        |
|-----------|-----------------------|---------------------------------------|
| `init`    | `commands/init.rs`    | Initializes a repository.             |
| `create`  | `commands/create.rs`  | Creates a bucket.                     |
| `commit`  | `commands/commit.rs`  | Handles commits to buckets.           |
| `status`  | `commands/status.rs`  | Checks the status of a repository.    |

### Utilities
| File                | Responsibility                                    |
|---------------------|--------------------------------------------------|
| `utils/checks.rs`   | Validates directories and repositories.           |
| `utils/mod.rs`      | Entry point for utility modules.                  |



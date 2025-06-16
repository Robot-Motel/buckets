# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## About Buckets

Buckets is a CLI tool for game asset and expectation management. It controls versions of work and sets/records expectations between collaborators. Each stage of the workflow is represented by a bucket containing resources to create game assets at specific production pipeline stages.

## Development Commands

### Building and Testing
- `cargo build` - Build the project
- `cargo build --release` - Build optimized release version  
- `cargo test` - Run all tests
- `cargo test --release` - Run tests in release mode
- `cargo clippy` - Run linting checks
- `cargo fmt` - Format code

### Advanced Testing
- `cargo llvm-cov nextest --all-features --workspace --lcov --output-path lcov.info` - Generate code coverage report (requires cargo-llvm-cov and cargo-nextest)

## Architecture Overview

### Command Structure
- Each CLI subcommand has a dedicated module in `src/commands/`
- Commands implement the `BucketCommand` trait with an `execute()` function
- Commands are defined in `args.rs` using clap and dispatched in `main.rs`

### Key Components
- **args.rs**: CLI argument parsing using clap
- **errors.rs**: Centralized error handling with `BucketError` enum
- **utils/**: Reusable utility functions (directory validation, checks, etc.)
- **data/**: Data structures for buckets and commits
- **world.rs**: Global state management

### Database & Storage
- Uses DuckDB for data persistence (see `src/sql/schema.sql`)
- File hashing with BLAKE3
- Compression support with zstd
- UUID-based object identification

### Thread-Local State
- `CURRENT_DIR`: Current working directory
- `EXIT`: Program exit code tracking

### Error Handling
All errors use the centralized `BucketError` enum with `From<io::Error>` for seamless propagation with `?` operator.

## Testing Structure
- Tests are in `tests/` directory with one file per command
- Uses `serial_test` for tests that need sequential execution
- Common test utilities in `tests/common.rs`
- Uses tempfile for isolated test environments
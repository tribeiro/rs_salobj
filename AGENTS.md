# Developer Agent Guidelines - rs_salobj

This repository contains the Rust implementation of the Object-Oriented Service Abstraction Layer (SAL), used for control software at the Vera C. Rubin Observatory.

## 🛠 Build, Lint, and Test Commands

The project is a Rust workspace. Commands should generally be run from the root.

### Build & Check
- **Build all:** `cargo build --workspace`
- **Check (fast):** `cargo check --workspace`
- **Clippy (lint):** `cargo clippy --workspace -- -D warnings`
- **Format:** `cargo fmt --all`

### Running Tests
- **Run all tests:** `cargo test --workspace`
- **Run single crate tests:** `cargo test -p salobj`
- **Run a single test module:** `cargo test -p salobj --lib domain::tests`
- **Run a specific test:** `cargo test -p salobj --lib domain::tests::get_default_identity -- --nocapture`
- **Run documentation tests:** `cargo test --doc`

## 🦀 Code Style & Conventions

### 1. General Principles
- Follow standard Rust idioms and naming conventions (`PascalCase` for types/enums, `snake_case` for functions/variables).
- Use `rustfmt` for all formatting.
- Prefer `async/await` using the `tokio` runtime.

### 2. Imports
- Group imports in the following order:
  1. Standard library (`std::...`)
  2. External crates
  3. Internal crate modules (`crate::...`)
- Use `use crate::...` for internal module imports within the same crate.

### 3. Error Handling
- Use the custom error system defined in `salobj/src/error/errors.rs`.
- All fallible functions should return `SalObjResult<T>`, which is an alias for `Result<T, SalObjError>`.
- Implement `From<ExternalErrorType>` for `SalObjError` in `errors.rs` to support the `?` operator.
- Avoid `unwrap()` and `expect()` in library code; prefer returning errors.

### 4. Async & Concurrency
- Use `tokio` for tasks, timers, and synchronization.
- Use `tokio::sync::mpsc` for multi-producer, single-consumer communication.
- Use `tokio::sync::watch` for state changes that need to be observed by multiple tasks.
- Background tasks should be spawned using `tokio::task::spawn`.

### 5. SAL Specific Patterns
- **Topics:** Use the `#[add_sal_topic_fields]` attribute and `#[derive(BaseSALTopic)]` on structs representing SAL topics.
- **Commands:** Use the `handle_command!` macro in `TestCSC` or similar controllers to implement the command processing loop.
- **CSCs:** Components should implement the `BaseCSC` trait and follow the state machine transitions (`Standby`, `Disabled`, `Enabled`, `Fault`, `Offline`).
- **Domain:** Use the `Domain` struct to manage Kafka client identities and configurations.

### 6. Documentation
- Use `//!` for module-level documentation.
- Use `///` for public struct, enum, and function documentation.
- Document the *why* and any side effects or background tasks spawned by a function.

## 📁 Project Structure

- `salobj/`: Main library implementation.
- `base_topic_derive/`: Procedural macros for deriving `BaseSALTopic`.
- `handle_command/`: Procedural macro for command handling logic.
- `ts_xml_define/`: Procedural macro for generating SAL telemetry Rust structs from XML schema definitions (independent crate).

## ⚠️ Safety & Best Practices
- **Kafka:** Ensure `LSST_KAFKA_BROKER_ADDR` and `LSST_SCHEMA_REGISTRY_URL` are handled via the `Domain` utility.
- **Cleanup:** Always ensure background tasks (like heartbeats or telemetry loops) are properly aborted or joined when a component is dropped or transitions to `Offline`.

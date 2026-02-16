# Plan - Initialize Rust Project

## Meta
**Goal**: Initialize the Rust project structure and dependencies required for the Slint Kanban application.

## Changes

### 1. `Cargo.toml`
-   **Add Dependencies**:
    -   `slint = "1.9.0"`
    -   `serde = { version = "1.0", features = ["derive"] }`
    -   `serde_yaml = "0.9"`
    -   `walkdir = "2"`
    -   `chrono = "0.4"`
    -   `anyhow = "1.0"`
-   **Add Build Dependencies**:
    -   `slint-build = "1.9.0"`

### 2. `build.rs`
-   Create a build script to compile Slint UI files.
-   Config: `slint_build::compile("ui/app.slint").unwrap();`

### 3. `ui/app.slint`
-   Create a basic UI definition.
-   Content: A simple `Window` with "Slint Kanban" title.

### 4. `src/main.rs`
-   Initialize Slint application.
-   Import generated `App` struct.
-   Run the app.

## Verification

### Automated Tests
-   Run `cargo check` to verify dependencies.
-   Run `cargo test` (trivial).

### Manual Verification
-   Run `cargo run`.
-   Verify that a window with title "Slint Kanban" appears.

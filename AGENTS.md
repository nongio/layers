# Development Workflow for layers

This repository is a Rust workspace. Continuous integration uses Rust 1.83.0 on Ubuntu.
Follow these steps before submitting a pull request.

## Formatting and Linting
- Format the code with rustfmt and ensure the check passes:
  ```bash
  cargo fmt --all -- --check
  ```
- Lint using Clippy. All warnings must be fixed:
  ```bash
  cargo clippy --features "default" -- -D warnings
  ```

## Building and Docs
- Build the workspace to ensure it compiles:
  ```bash
  cargo check --features "default"
  ```
- Build the API documentation the same way CI does:
  ```bash
  RUSTDOCFLAGS=--cfg=docsrs cargo doc --no-deps --features "default" -p lay-rs
  ```

## Criterion Benchmarks
- Benchmarks can be executed with Criterion:
  ```bash
  cargo bench --bench my_benchmark
  ```
  CI compares benchmark results against the base branch.

## System Dependencies
CI installs several development packages for Skia and Wayland support:
`libdrm-dev libudev-dev libgbm-dev libxkbcommon-dev libegl1-mesa-dev libwayland-dev libinput-dev libdbus-1-dev libsystemd-dev libseat-dev`.
Ensure these packages are available when running the above commands locally.


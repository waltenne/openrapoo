# Contributing to OpenRapoo

Thank you for your interest in contributing to OpenRapoo! We welcome bug reports, hardware telemetry logs, documentation improvements, and code contributions.

---

## 🛠️ Development Setup

### Prerequisites

Ensure you have Rust (1.75+) installed via `rustup` and the required C libraries:

```bash
# Ubuntu / Debian
sudo apt install build-essential pkg-config libhidapi-dev libudev-dev libevdev-dev libdbus-1-dev desktop-file-utils
```

### Checking & Building Workspace

```bash
# Format check
cargo fmt --all -- --check

# Workspace check
cargo check --workspace

# Clippy lints
cargo clippy --workspace --all-targets -- -D warnings

# Run tests
cargo test --workspace

# Build release
cargo build --release --workspace
```

---

## 🐞 Submitting Bug & Hardware Telemetry Reports

If you own a Rapoo MT760 Pro or other Rapoo peripheral, you can contribute hardware logs without revealing personal information:

1. Run the diagnostic tool:
   ```bash
   cargo run --bin openrapoo-diag -- generate-report
   ```
2. The report is saved to `~/.local/share/openrapoo/report-YYYY-MM-DD.md`.
3. Open a GitHub Issue attaching the generated diagnostic report.

---

## 🤖 AI Assistance Policy

OpenRapoo welcomes pull requests written with the assistance of AI tools. However, please ensure that:
1. All PRs are manually tested on Linux hardware or with unit tests.
2. Code follows workspace Rust idioms and passes `cargo clippy -- -D warnings`.
3. No hallucinated features or non-existent hardware protocols are presented as supported.

---

## 📜 Pull Request Process

1. Fork the repository and create a feature branch (`git checkout -b feature/my-feature`).
2. Commit your changes with clear messages.
3. Verify that `cargo fmt`, `cargo check`, `cargo clippy`, and `cargo test` pass cleanly.
4. Push your branch and open a Pull Request against `main`.

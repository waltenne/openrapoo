# OpenRapoo Development Guide

This guide covers building, testing, linting, adding i18n keys, and packaging OpenRapoo locally.

---

## 🛠️ Development Environment Setup

Ensure you have Rust (1.75+) installed via `rustup`:

```bash
rustup update stable
```

Install C development dependencies (Ubuntu/Debian):

```bash
sudo apt install build-essential pkg-config libhidapi-dev libudev-dev libevdev-dev libdbus-1-dev desktop-file-utils
```

---

## 💻 Common Cargo Commands

```bash
# Code formatting check
cargo fmt --all -- --check

# Workspace check
cargo check --workspace

# Clippy lints
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Execute all unit and integration tests
cargo test --workspace

# Compile optimized release binaries
cargo build --release --workspace
```

---

## 🌐 Adding New i18n Translation Keys

All UI translations are managed in `crates/openrapoo-gui/src/i18n.rs`. To add a new string:

1. Add a domain helper function taking `lang: Language`:
   ```rust
   pub fn my_new_label(lang: Language) -> &'static str {
       match lang {
           Language::English => "My New Label",
           Language::Portuguese => "Meu Novo Rótulo",
       }
   }
   ```
2. Reference `i18n::my_new_label(language)` in your GPUI component.
3. Run `cargo test --workspace` to verify coverage tests pass.

---

## 📦 Local Packaging Suite Execution

To generate `.deb`, `.AppImage`, and `.tar.gz` packages locally:

```bash
./packaging/build-all.sh
```

Output files are created in the `dist/` directory.


# AI-Assisted Development Disclosure

OpenRapoo is developed with the assistance of artificial intelligence tools. AI is used as a supporting tool for research, implementation, architecture, code generation and review, testing, troubleshooting, and documentation. Technical decisions, code reviews, hardware validations, and ultimate code responsibility remain with the project maintainers.

---

## 🔍 How AI Tools Are Used in OpenRapoo

1. **Hardware Protocol & Bus Investigation**:
   - Assisting in analyzing HID descriptors, evdev event structs, UPower D-Bus interfaces, and BlueZ D-Bus properties.
2. **Code Generation & Refactoring**:
   - Generating idiomatic Rust implementations across workspace crates (`openrapoo-core`, `openrapoo-daemon`, `openrapoo-gui`, `openrapoo-diag`).
   - Building GPUI UI component views and internationalization (i18n) catalogs.
3. **Automated Testing Suite**:
   - Creating comprehensive unit and integration test suites (`cargo test --workspace`) covering telemetry, IPC message parsing, and settings persistence.
4. **CI/CD & Packaging Suite**:
   - Setting up GitHub Actions workflows (`ci.yml`, `release.yml`), Debian `.deb` package layouts, and AppImage scripts.
5. **Documentation**:
   - Authoring and synchronizing bilingual documentation trees in English and Brazilian Portuguese.

---

## 🛡️ Human Verification & Quality Safeguards

- **No Blind Acceptance**: AI code suggestions are evaluated against Rust compiler safety rules, `cargo clippy`, and manual code inspection.
- **Hardware Validation**: Real-world hardware behavior (Rapoo MT760 Pro physical buttons, HID report dumps, USB/Bluetooth multi-transport switching) is verified on Linux hardware.
- **Security Scoping**: Security-sensitive scripts (UDev rules, systemd user units, permissions) are reviewed to prevent unprivileged root escalation risks.


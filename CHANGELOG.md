# Changelog

All notable changes to OpenRapoo will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-09-19

### Added
- **GPUI Interface**: High-performance native desktop GUI with centered tab bar (Buttons, Pointer & Performance, Device Info, Diagnostics).
- **Internationalization (i18n)**: Instant switching and persistence between English (`en-US`) and Brazilian Portuguese (`pt-BR`) saved in `$XDG_CONFIG_HOME/openrapoo/settings.json`.
- **DPI & Polling Rate Selection**: UI controls for 800, 1000, 1200, 1600, 2400, 3200, 4000 DPI and 125, 250, 500, 1000 Hz polling rates.
- **Button Remapping Daemon**: `openrapoo-daemon` background service remapping mouse buttons via `evdev`/`uinput`.
- **Multi-Provider Battery Telemetry**: Aggregate battery reader supporting UPower, BlueZ D-Bus, and HID raw buses without dummy values.
- **Packaging Suite**: Native Debian `.deb` builder, standalone `AppImage` builder, release tarball generator, and SHA-256 checksum automation.
- **GitHub Actions CI/CD**: Automated testing matrix on Ubuntu 22.04 and 24.04, clippy, fmt, desktop validation, and tag-triggered GitHub Release publishing.


# Security Policy for OpenRapoo

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

---

## Security Model & Non-Root Design Rationale

OpenRapoo is designed with Linux security best practices:
1. **No Root GUI**: The Graphical Interface (`openrapoo-gui`) executes under standard unprivileged user credentials. Running `openrapoo-gui` via `sudo` is strictly discouraged.
2. **Scoped Hardware Access**: Access to `/dev/hidraw*` and `/dev/input/event*` nodes is granted safely through explicit UDev rules (`udev/99-openrapoo.rules`) limited to Rapoo Vendor IDs (`0x24AE`) and matching hardware attributes.
3. **Daemon Isolation**: `openrapoo-daemon` executes as a systemd user service (`/usr/lib/systemd/user/openrapoo-daemon.service`) within user session bounds.

---

## Reporting a Vulnerability

If you discover a potential security vulnerability in OpenRapoo, please report it by opening a private security disclosure on GitHub or emailing `openrapoo@gmail.com`.

Please include:
- A description of the issue.
- Instructions to reproduce the vulnerability.
- Any relevant logs or environment details.


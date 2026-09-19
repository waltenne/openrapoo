# OpenRapoo Troubleshooting Guide

Common issues, permission diagnostic checks, and resolution procedures.

---

## 🔍 Permission Issues (`Permission Denied` on `/dev/input/event*` or `/dev/hidraw*`)

### Cause
Your user account lacks permission to read/write hardware input device nodes.

### Resolution
1. Verify `input` group membership:
   ```bash
   groups $USER
   ```
2. If `input` is missing, add your user:
   ```bash
   sudo usermod -aG input $USER
   ```
3. Re-install UDev rules:
   ```bash
   sudo cp udev/99-openrapoo.rules /etc/udev/rules.d/
   sudo udevadm control --reload-rules
   sudo udevadm trigger
   ```
4. Log out and log back in.

---

## ⚡ Daemon Connection Warning

### Cause
`openrapoo-gui` cannot connect to `openrapoo-daemon` Unix socket (`$XDG_RUNTIME_DIR/openrapoo.sock`).

### Resolution
1. Check if daemon process is running:
   ```bash
   pgrep -fl openrapoo-daemon
   ```
2. Manually start daemon to inspect log output:
   ```bash
   openrapoo-daemon
   ```

---

## 🔋 Battery Telemetry Reading Shows `Unavailable` or `Stale`

### Cause
- USB Cable Mode: Cable transport powers the mouse directly and does not broadcast wireless telemetry reports.
- Bluetooth Mode: Telemetry is provided via BlueZ D-Bus (`org.bluez.Battery1`). If BlueZ has not queried the GATT battery service recently, the status is marked as `Stale`.

### Resolution
Press **Refresh Battery Now** in the Battery Diagnostics tab to trigger a active bus scan.


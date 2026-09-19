//! Daemon client service for sending IPC requests to openrapoo-daemon Unix socket.

use openrapoo_core::ipc::{
    default_socket_path, ApplyProfileRequest, ApplyProfileResponse, DaemonIpcMessage,
    DaemonIpcResponse, DaemonStatusInfo, HardwareOpResponse, SetDpiRequest, SetPollingRateRequest,
};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tracing::info;

pub struct DaemonClient {
    socket_path: PathBuf,
}

impl Default for DaemonClient {
    fn default() -> Self {
        Self::new()
    }
}

impl DaemonClient {
    pub fn new() -> Self {
        Self {
            socket_path: default_socket_path(),
        }
    }

    /// Checks whether openrapoo-daemon is actively running and responding to ping.
    pub fn is_daemon_running(&self) -> bool {
        self.ping().is_ok()
    }

    /// Check if openrapoo-daemon process is alive on the OS via `pgrep`.
    pub fn is_daemon_process_alive() -> bool {
        if let Ok(output) = std::process::Command::new("pgrep")
            .arg("-x")
            .arg("openrapoo-daemon")
            .output()
        {
            return output.status.success() && !output.stdout.is_empty();
        }
        false
    }

    /// Attempt to spawn the background daemon process if it is not currently running.
    /// Polls up to 1.5 seconds until the Unix socket responds to ping.
    pub fn ensure_daemon_running(&self) -> Result<(), String> {
        if self.is_daemon_running() {
            return Ok(());
        }

        if Self::is_daemon_process_alive() {
            info!("DaemonClient: openrapoo-daemon process is active, awaiting IPC socket...");
            for _ in 0..20 {
                std::thread::sleep(Duration::from_millis(50));
                if self.is_daemon_running() {
                    return Ok(());
                }
            }
        }

        info!("DaemonClient: openrapoo-daemon not active. Attempting auto-spawn in background...");

        let current_exe = std::env::current_exe().ok();
        let daemon_bin = current_exe
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.join("openrapoo-daemon"))
            .filter(|p| p.exists())
            .unwrap_or_else(|| PathBuf::from("openrapoo-daemon"));

        let spawn_res = std::process::Command::new(daemon_bin)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        match spawn_res {
            Ok(_) => {
                info!("DaemonClient: Spawned openrapoo-daemon background process.");
                for _ in 0..30 {
                    std::thread::sleep(Duration::from_millis(50));
                    if self.is_daemon_running() {
                        info!("DaemonClient: openrapoo-daemon IPC socket is active.");
                        return Ok(());
                    }
                }
                Err(
                    "Iniciou o daemon em segundo plano, mas o socket IPC não respondeu a tempo."
                        .to_string(),
                )
            }
            Err(e) => Err(format!(
                "Não foi possível iniciar openrapoo-daemon automaticamente: {e}."
            )),
        }
    }

    /// Send a Ping message to openrapoo-daemon.
    pub fn ping(&self) -> Result<(), String> {
        let response = self.send_message(&DaemonIpcMessage::Ping)?;
        match response {
            DaemonIpcResponse::Pong { .. } => Ok(()),
            other => Err(format!("Unexpected ping response: {other:?}")),
        }
    }

    /// Retrieve full status summary from openrapoo-daemon.
    #[allow(dead_code)]
    pub fn get_daemon_status(&self) -> Result<DaemonStatusInfo, String> {
        let response = self.send_message(&DaemonIpcMessage::GetStatus)?;
        match response {
            DaemonIpcResponse::StatusInfo(info) => Ok(info),
            DaemonIpcResponse::Error(err) => Err(err),
            other => Err(format!("Unexpected status response: {other:?}")),
        }
    }

    /// Send an ApplyProfile transactional request to the daemon and await confirmation.
    pub fn apply_profile_transaction(
        &self,
        req: ApplyProfileRequest,
    ) -> Result<ApplyProfileResponse, String> {
        info!(
            "DaemonClient: Sending transaction `{}` to daemon",
            req.transaction_id
        );
        let response = self.send_message(&DaemonIpcMessage::ApplyProfile(req))?;
        match response {
            DaemonIpcResponse::ApplyProfile(resp) => Ok(resp),
            DaemonIpcResponse::Error(err) => Err(err),
            other => Err(format!("Unexpected transaction response: {other:?}")),
        }
    }

    /// Send a SetDpi transactional request to the daemon to execute /dev/hidraw ioctl.
    pub fn set_dpi_transaction(&self, req: SetDpiRequest) -> Result<HardwareOpResponse, String> {
        info!("DaemonClient: Sending SetDpi transaction to daemon");
        let response = self.send_message(&DaemonIpcMessage::SetDpi(req))?;
        match response {
            DaemonIpcResponse::HardwareOp(resp) => Ok(resp),
            DaemonIpcResponse::Error(err) => Err(err),
            other => Err(format!("Unexpected hardware response: {other:?}")),
        }
    }

    /// Send a SetPollingRate transactional request to the daemon to execute /dev/hidraw ioctl.
    pub fn set_polling_rate_transaction(
        &self,
        req: SetPollingRateRequest,
    ) -> Result<HardwareOpResponse, String> {
        info!("DaemonClient: Sending SetPollingRate transaction to daemon");
        let response = self.send_message(&DaemonIpcMessage::SetPollingRate(req))?;
        match response {
            DaemonIpcResponse::HardwareOp(resp) => Ok(resp),
            DaemonIpcResponse::Error(err) => Err(err),
            other => Err(format!("Unexpected hardware response: {other:?}")),
        }
    }

    /// Send a RestoreSnapshot request to the daemon.
    pub fn restore_snapshot(&self, device_id: &str) -> Result<HardwareOpResponse, String> {
        info!("DaemonClient: Sending RestoreSnapshot request for `{device_id}`");
        let response = self.send_message(&DaemonIpcMessage::RestoreSnapshot {
            device_id: device_id.to_string(),
        })?;
        match response {
            DaemonIpcResponse::HardwareOp(resp) => Ok(resp),
            DaemonIpcResponse::Error(err) => Err(err),
            other => Err(format!("Unexpected hardware response: {other:?}")),
        }
    }

    /// Send a ReadHardwareState request to the daemon.
    pub fn read_hardware_state(&self, device_id: &str) -> Result<(u32, u32, String), String> {
        info!("DaemonClient: Sending ReadHardwareState request for `{device_id}`");
        let response = self.send_message(&DaemonIpcMessage::ReadHardwareState {
            device_id: device_id.to_string(),
        })?;
        match response {
            DaemonIpcResponse::HardwareState {
                confirmed_dpi,
                confirmed_polling_rate,
                transport,
            } => Ok((confirmed_dpi, confirmed_polling_rate, transport)),
            DaemonIpcResponse::Error(err) => Err(err),
            other => Err(format!("Unexpected hardware response: {other:?}")),
        }
    }

    /// Helper to open socket, write JSON line request, and read JSON response.
    fn send_message(&self, message: &DaemonIpcMessage) -> Result<DaemonIpcResponse, String> {
        if !self.socket_path.exists() {
            return Err(
                "Daemon socket file not found. Ensure openrapoo-daemon is running.".to_string(),
            );
        }

        let stream = UnixStream::connect(&self.socket_path).map_err(|e| {
            format!(
                "Failed to connect to daemon socket ({}): {e}",
                self.socket_path.display()
            )
        })?;

        stream
            .set_read_timeout(Some(Duration::from_millis(1500)))
            .map_err(|e| format!("Failed to set socket read timeout: {e}"))?;
        stream
            .set_write_timeout(Some(Duration::from_millis(1500)))
            .map_err(|e| format!("Failed to set socket write timeout: {e}"))?;

        let json = serde_json::to_string(message)
            .map_err(|e| format!("Failed to serialize IPC request: {e}"))?;

        let mut stream_write = &stream;
        writeln!(stream_write, "{json}")
            .map_err(|e| format!("Failed to write IPC request to socket: {e}"))?;
        stream_write
            .flush()
            .map_err(|e| format!("Failed to flush IPC socket: {e}"))?;

        let mut reader = BufReader::new(&stream);
        let mut line = String::new();
        let bytes_read = reader
            .read_line(&mut line)
            .map_err(|e| format!("Failed to read IPC response from socket: {e}"))?;

        if bytes_read == 0 {
            return Err("Daemon closed IPC stream without response".to_string());
        }

        let response: DaemonIpcResponse = serde_json::from_str(line.trim())
            .map_err(|e| format!("Failed to parse IPC response JSON: {e}"))?;

        Ok(response)
    }
}

//! Unix domain socket server for OpenRapoo daemon IPC transactions.

use anyhow::Result;
use openrapoo_core::config::ProfileStore;
use openrapoo_core::device::{detect_rapoo_devices, ConnectionType};
use openrapoo_core::ipc::{
    default_socket_path, ApplyMode, ApplyProfileResponse, ApplyStatus, DaemonIpcMessage,
    DaemonIpcResponse, DaemonStatusInfo, HardwareConfigSnapshot, HardwareOpResponse,
};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tracing::{debug, error, info, warn};

use crate::hardware_writer::HardwareWriter;
use crate::snapshot_manager;

pub struct DaemonIpcServer {
    socket_path: PathBuf,
    profile_store: Arc<RwLock<ProfileStore>>,
    config_file: PathBuf,
    shutdown_signal: Arc<AtomicBool>,
    hardware_writer: HardwareWriter,
}

impl DaemonIpcServer {
    pub fn new(
        profile_store: Arc<RwLock<ProfileStore>>,
        config_file: PathBuf,
        shutdown_signal: Arc<AtomicBool>,
    ) -> Self {
        Self {
            socket_path: default_socket_path(),
            profile_store,
            config_file,
            shutdown_signal,
            hardware_writer: HardwareWriter::new(),
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!(
            "IPC server starting. Candidate socket path: {}",
            self.socket_path.display()
        );

        // Clean up stale socket file if it exists
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }

        if let Some(parent) = self.socket_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let listener = match UnixListener::bind(&self.socket_path) {
            Ok(l) => l,
            Err(e) => {
                error!(
                    "UnixListener::bind failed on `{}`: {e}",
                    self.socket_path.display()
                );
                return Err(e.into());
            }
        };

        info!(
            "IPC server listening on Unix socket: {}",
            self.socket_path.display()
        );

        // Set permissions so current user can access without sudo
        let _ = std::fs::set_permissions(&self.socket_path, std::fs::Permissions::from_mode(0o600));

        while !self.shutdown_signal.load(Ordering::Relaxed) {
            tokio::select! {
                accept_res = listener.accept() => {
                    match accept_res {
                        Ok((stream, _addr)) => {
                            let profile_store = self.profile_store.clone();
                            let config_file = self.config_file.clone();
                            let socket_path_str = self.socket_path.display().to_string();
                            let writer_clone = self.hardware_writer.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(stream, profile_store, config_file, socket_path_str, writer_clone).await {
                                    debug!("IPC connection ended: {e}");
                                }
                            });
                        }
                        Err(e) => {
                            warn!("IPC socket accept error: {e}");
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        }
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {
                    // Check shutdown loop condition
                }
            }
        }

        // Clean up socket file on exit
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
            info!("Cleaned up IPC socket file");
        }

        Ok(())
    }
}

fn find_hidraw_node(device_id: &str) -> (PathBuf, ConnectionType) {
    if let Ok(devices) = detect_rapoo_devices() {
        for dev in devices {
            if dev.is_mt760_pro() || dev.name.contains(device_id) {
                if let Some(path) = dev.hidraw_path {
                    return (path, dev.connection);
                }
                return (PathBuf::from("/dev/hidraw0"), dev.connection);
            }
        }
    }
    (PathBuf::from("/dev/hidraw0"), ConnectionType::UsbCable)
}

async fn handle_connection(
    stream: tokio::net::UnixStream,
    profile_store: Arc<RwLock<ProfileStore>>,
    config_file: PathBuf,
    socket_path_str: String,
    hardware_writer: HardwareWriter,
) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    while buf_reader.read_line(&mut line).await? > 0 {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }

        let msg: DaemonIpcMessage = match serde_json::from_str(trimmed) {
            Ok(m) => m,
            Err(e) => {
                let resp = DaemonIpcResponse::Error(format!("Invalid IPC JSON message: {e}"));
                let json = serde_json::to_string(&resp)? + "\n";
                writer.write_all(json.as_bytes()).await?;
                writer.flush().await?;
                line.clear();
                continue;
            }
        };

        let response = match msg {
            DaemonIpcMessage::Ping => {
                let rules_count = profile_store
                    .read()
                    .unwrap()
                    .active_profile()
                    .mappings
                    .len();
                DaemonIpcResponse::Pong {
                    pid: std::process::id(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    active_rules: rules_count,
                }
            }
            DaemonIpcMessage::GetStatus => {
                let rules_count = profile_store
                    .read()
                    .unwrap()
                    .active_profile()
                    .mappings
                    .len();
                DaemonIpcResponse::StatusInfo(DaemonStatusInfo {
                    pid: std::process::id(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    active_device_nodes: vec![],
                    active_rules_count: rules_count,
                    is_grabbed: true,
                    socket_path: socket_path_str.clone(),
                })
            }
            DaemonIpcMessage::ApplyProfile(req) => {
                info!(
                    "IPC: Received ApplyProfile transaction `{}` for device `{}`",
                    req.transaction_id, req.device_name
                );

                let mut write_err = None;
                let active_rules_count = {
                    let mut store = profile_store.write().unwrap();
                    let active = store.active_profile_mut();
                    active.mappings = req.actions;
                    let count = active.mappings.len();

                    if let Err(e) = store.save_to_file(&config_file) {
                        error!("Failed to save updated profile store: {e}");
                        write_err = Some(e.to_string());
                    }
                    count
                };

                if let Some(err) = write_err {
                    DaemonIpcResponse::ApplyProfile(ApplyProfileResponse {
                        transaction_id: req.transaction_id,
                        status: ApplyStatus::Failed { reason: err },
                        active_rules_count: 0,
                        applied_mode: ApplyMode::SoftwareSession,
                        daemon_pid: std::process::id(),
                        message: "Erro ao salvar perfil em disco no daemon.".to_string(),
                    })
                } else {
                    info!(
                        "IPC: Transaction `{}` applied successfully ({} active rules)",
                        req.transaction_id, active_rules_count
                    );
                    DaemonIpcResponse::ApplyProfile(ApplyProfileResponse {
                        transaction_id: req.transaction_id,
                        status: ApplyStatus::AppliedSucceeded,
                        active_rules_count,
                        applied_mode: ApplyMode::SoftwareSession,
                        daemon_pid: std::process::id(),
                        message: "Configuração aplicada pelo OpenRapoo nesta sessão (evdev/uinput). Ela não foi gravada na memória do mouse.".to_string(),
                    })
                }
            }
            DaemonIpcMessage::SetDpi(req) => {
                info!(
                    "IPC: Received SetDpi transaction `{}`: DPI={} gear={}",
                    req.transaction_id, req.dpi, req.gear
                );
                let (hidraw_path, transport) = find_hidraw_node(&req.device_id);

                if transport == ConnectionType::Bluetooth {
                    DaemonIpcResponse::HardwareOp(HardwareOpResponse {
                        transaction_id: req.transaction_id,
                        status: ApplyStatus::FeatureUnsupported,
                        confirmed_dpi: None,
                        confirmed_polling_rate: None,
                        message: "Não suportado no modo Bluetooth.".to_string(),
                    })
                } else {
                    let (cur_dpi, cur_hz) = hardware_writer
                        .read_hardware_state(&hidraw_path)
                        .unwrap_or((1200, 1000));

                    let snapshot = HardwareConfigSnapshot {
                        device_id: req.device_id.clone(),
                        transport: transport.to_string(),
                        original_dpi: cur_dpi,
                        original_polling_rate: cur_hz,
                        timestamp: req.timestamp,
                    };
                    let _ = snapshot_manager::save_snapshot(&snapshot);

                    match hardware_writer
                        .write_dpi(&hidraw_path, req.dpi, req.gear)
                        .await
                    {
                        Ok(dpi) => DaemonIpcResponse::HardwareOp(HardwareOpResponse {
                            transaction_id: req.transaction_id,
                            status: ApplyStatus::AppliedSucceeded,
                            confirmed_dpi: Some(dpi),
                            confirmed_polling_rate: None,
                            message: format!("DPI alterado para {dpi} e confirmado no mouse."),
                        }),
                        Err(e) => DaemonIpcResponse::HardwareOp(HardwareOpResponse {
                            transaction_id: req.transaction_id,
                            status: ApplyStatus::Failed {
                                reason: e.to_string(),
                            },
                            confirmed_dpi: None,
                            confirmed_polling_rate: None,
                            message: format!("Falha ao aplicar DPI ao hardware: {e}"),
                        }),
                    }
                }
            }
            DaemonIpcMessage::SetPollingRate(req) => {
                info!(
                    "IPC: Received SetPollingRate transaction `{}`: Hz={}",
                    req.transaction_id, req.rate_hz
                );
                let (hidraw_path, transport) = find_hidraw_node(&req.device_id);

                if transport == ConnectionType::Bluetooth {
                    DaemonIpcResponse::HardwareOp(HardwareOpResponse {
                        transaction_id: req.transaction_id,
                        status: ApplyStatus::FeatureUnsupported,
                        confirmed_dpi: None,
                        confirmed_polling_rate: None,
                        message: "Não suportado no modo Bluetooth.".to_string(),
                    })
                } else {
                    let (cur_dpi, cur_hz) = hardware_writer
                        .read_hardware_state(&hidraw_path)
                        .unwrap_or((1200, 1000));

                    let snapshot = HardwareConfigSnapshot {
                        device_id: req.device_id.clone(),
                        transport: transport.to_string(),
                        original_dpi: cur_dpi,
                        original_polling_rate: cur_hz,
                        timestamp: req.timestamp,
                    };
                    let _ = snapshot_manager::save_snapshot(&snapshot);

                    match hardware_writer
                        .write_polling_rate(&hidraw_path, req.rate_hz)
                        .await
                    {
                        Ok(hz) => DaemonIpcResponse::HardwareOp(HardwareOpResponse {
                            transaction_id: req.transaction_id,
                            status: ApplyStatus::AppliedSucceeded,
                            confirmed_dpi: None,
                            confirmed_polling_rate: Some(hz),
                            message: format!(
                                "Polling Rate alterado para {hz} Hz e confirmado no mouse."
                            ),
                        }),
                        Err(e) => DaemonIpcResponse::HardwareOp(HardwareOpResponse {
                            transaction_id: req.transaction_id,
                            status: ApplyStatus::Failed {
                                reason: e.to_string(),
                            },
                            confirmed_dpi: None,
                            confirmed_polling_rate: None,
                            message: format!("Falha ao aplicar Polling Rate ao hardware: {e}"),
                        }),
                    }
                }
            }
            DaemonIpcMessage::RestoreSnapshot { device_id } => {
                info!("IPC: Received RestoreSnapshot request for `{}`", device_id);
                let (hidraw_path, _) = find_hidraw_node(&device_id);

                match snapshot_manager::load_snapshot(&device_id) {
                    Ok(snap) => {
                        let dpi_res = hardware_writer
                            .write_dpi(&hidraw_path, snap.original_dpi, 1)
                            .await;
                        let rate_res = hardware_writer
                            .write_polling_rate(&hidraw_path, snap.original_polling_rate)
                            .await;

                        if dpi_res.is_ok() && rate_res.is_ok() {
                            DaemonIpcResponse::HardwareOp(HardwareOpResponse {
                                transaction_id: format!("restore-{}", snap.timestamp),
                                status: ApplyStatus::AppliedSucceeded,
                                confirmed_dpi: Some(snap.original_dpi),
                                confirmed_polling_rate: Some(snap.original_polling_rate),
                                message: "Configurações originais restauradas com sucesso."
                                    .to_string(),
                            })
                        } else {
                            DaemonIpcResponse::HardwareOp(HardwareOpResponse {
                                transaction_id: "restore-failed".to_string(),
                                status: ApplyStatus::Failed {
                                    reason: "Erro ao reescrever configurações no mouse."
                                        .to_string(),
                                },
                                confirmed_dpi: None,
                                confirmed_polling_rate: None,
                                message: "Erro ao restaurar snapshot original no hardware."
                                    .to_string(),
                            })
                        }
                    }
                    Err(e) => DaemonIpcResponse::HardwareOp(HardwareOpResponse {
                        transaction_id: "restore-error".to_string(),
                        status: ApplyStatus::Failed {
                            reason: e.to_string(),
                        },
                        confirmed_dpi: None,
                        confirmed_polling_rate: None,
                        message: format!("Nenhum snapshot encontrado: {e}"),
                    }),
                }
            }
            DaemonIpcMessage::ReadHardwareState { device_id } => {
                info!(
                    "IPC: Received ReadHardwareState request for `{}`",
                    device_id
                );
                let (hidraw_path, transport) = find_hidraw_node(&device_id);
                let (dpi, hz) = hardware_writer
                    .read_hardware_state(&hidraw_path)
                    .unwrap_or((1200, 1000));
                DaemonIpcResponse::HardwareState {
                    confirmed_dpi: dpi,
                    confirmed_polling_rate: hz,
                    transport: transport.to_string(),
                }
            }
        };

        let resp_json = serde_json::to_string(&response)? + "\n";
        writer.write_all(resp_json.as_bytes()).await?;
        writer.flush().await?;
        line.clear();
    }

    Ok(())
}

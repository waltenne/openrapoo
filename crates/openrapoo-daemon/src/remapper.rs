//! Event interceptor and remapping engine for OpenRapoo daemon.

use crate::virtual_device::OpenRapooVirtualDevice;
use anyhow::{Context, Result};
use evdev::{Device, EventType, InputEvent, Key};
use openrapoo_core::{
    config::{ButtonAction, MouseButton, ProfileStore},
    device::{detect_rapoo_devices, DeviceType},
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use tokio::io::unix::AsyncFd;
use tracing::{debug, error, info, warn};

/// Remapping engine configuration.
pub struct RemapperConfig {
    pub device_path: Option<PathBuf>,
    #[allow(dead_code)]
    pub config_path: Option<PathBuf>,
    pub dry_run: bool,
}

/// Main remapping service instance.
pub struct Remapper {
    config: RemapperConfig,
    profile_store: Arc<RwLock<ProfileStore>>,
    virtual_dev: Option<OpenRapooVirtualDevice>,
    shutdown_signal: Arc<AtomicBool>,
}

impl Remapper {
    /// Create a new Remapper instance.
    pub fn new(
        config: RemapperConfig,
        profile_store: Arc<RwLock<ProfileStore>>,
        shutdown_signal: Arc<AtomicBool>,
    ) -> Result<Self> {
        let virtual_dev = if config.dry_run {
            info!("Running in DRY-RUN mode — virtual device will not be created.");
            None
        } else {
            Some(OpenRapooVirtualDevice::new().context("Failed to create uinput virtual device")?)
        };

        Ok(Remapper {
            config,
            profile_store,
            virtual_dev,
            shutdown_signal,
        })
    }

    /// Run the main remapping event loop with automatic hotplug & reconnect support.
    pub async fn run(&mut self) -> Result<()> {
        info!("OpenRapoo daemon service active.");

        while !self.shutdown_signal.load(Ordering::Relaxed) {
            let evdev_paths = match self.resolve_device_paths() {
                Ok(paths) => paths,
                Err(e) => {
                    debug!("Device discovery failed: {e}");
                    Vec::new()
                }
            };

            if evdev_paths.is_empty() {
                info!("Waiting for Rapoo MT760 Pro mouse to be connected...");
                tokio::time::sleep(std::time::Duration::from_secs(4)).await;
                continue;
            }

            info!(
                "Discovered {} Rapoo mouse input node(s). Opening devices...",
                evdev_paths.len()
            );

            let mut devices = Vec::new();
            for path in &evdev_paths {
                match Device::open(path) {
                    Ok(mut dev) => {
                        let name = dev.name().unwrap_or("Unknown").to_string();
                        let name_lower = name.to_lowercase();
                        let is_keyboard_node = name_lower.contains("keyboard")
                            || name_lower.contains("teclado")
                            || name_lower.contains("e9050")
                            || name_lower.contains("kbd");

                        if is_keyboard_node {
                            info!(
                                "Device {} ({name}) is a keyboard node — leaving ungrabbed for system typing.",
                                path.display()
                            );
                        } else if !self.config.dry_run {
                            if let Err(e) = dev.grab() {
                                warn!(
                                    "Could not grab device {}: {e}. Events may leak to OS.",
                                    path.display()
                                );
                            } else {
                                info!("Grabbed exclusive access on {} ({name})", path.display());
                            }
                        } else {
                            info!(
                                "DRY-RUN: Inspecting {} ({name}) without grab",
                                path.display()
                            );
                        }
                        devices.push((path.clone(), dev));
                    }
                    Err(e) => {
                        warn!("Failed to open evdev node {}: {e}", path.display());
                    }
                }
            }

            if devices.is_empty() {
                warn!("No Rapoo mouse input nodes could be opened. Retrying in 5 seconds...");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }

            let (tx, mut rx) = tokio::sync::mpsc::channel::<(PathBuf, Vec<InputEvent>)>(100);
            let shutdown_signal = self.shutdown_signal.clone();
            let dry_run = self.config.dry_run;

            for (path, dev) in devices {
                let tx = tx.clone();
                let shutdown = shutdown_signal.clone();
                tokio::spawn(async move {
                    let mut async_dev = match AsyncFd::new(dev) {
                        Ok(ad) => ad,
                        Err(e) => {
                            warn!("Failed to create AsyncFd for {}: {e}", path.display());
                            return;
                        }
                    };

                    while !shutdown.load(Ordering::Relaxed) {
                        let mut guard = match async_dev.readable_mut().await {
                            Ok(g) => g,
                            Err(_) => break,
                        };

                        let events_res = guard
                            .get_inner_mut()
                            .fetch_events()
                            .map(|e| e.collect::<Vec<_>>());

                        match events_res {
                            Ok(ev_vec) => {
                                if !ev_vec.is_empty()
                                    && tx.send((path.clone(), ev_vec)).await.is_err()
                                {
                                    break;
                                }
                                guard.retain_ready();
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                guard.clear_ready();
                            }
                            Err(e) => {
                                warn!("Device {} disconnected or error: {e}", path.display());
                                break;
                            }
                        }
                    }

                    if !dry_run {
                        let _ = async_dev.into_inner().ungrab();
                        info!("Released grab on {}", path.display());
                    }
                });
            }
            drop(tx); // Drop local tx so rx closes when all device tasks exit

            // Main event dispatch loop (0% CPU when idle, epoll-driven)
            while let Some((path, events)) = rx.recv().await {
                if self.shutdown_signal.load(Ordering::Relaxed) {
                    break;
                }
                self.process_events(&path, &events);
            }

            if !self.shutdown_signal.load(Ordering::Relaxed) {
                info!(
                    "Rapoo mouse disconnected or endpoint closed. Will attempt reconnect in 3s..."
                );
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
        }

        info!("Remapping engine stopped cleanly.");
        Ok(())
    }

    /// Resolve target evdev device paths. Strictly filters for DeviceType::Mouse.
    fn resolve_device_paths(&self) -> Result<Vec<PathBuf>> {
        if let Some(ref path) = self.config.device_path {
            return Ok(vec![path.clone()]);
        }

        let detected = detect_rapoo_devices()?;
        let mut paths = Vec::new();
        for d in detected {
            if d.device_type == DeviceType::Mouse {
                if let Some(p) = d.evdev_path {
                    if !paths.contains(&p) {
                        paths.push(p);
                    }
                }
            } else {
                info!(
                    "Daemon skipping non-mouse Rapoo device '{}' ({}) from exclusive remapper grab",
                    d.name, d.device_type
                );
            }
        }

        Ok(paths)
    }

    /// Process a batch of raw evdev events from a physical device.
    fn process_events(&mut self, source_path: &Path, events: &[InputEvent]) {
        let active_profile = self.profile_store.read().unwrap().active_profile().clone();

        for event in events {
            let ev_type = event.event_type();
            let code = event.code();
            let value = event.value();

            // 1. Key & Button events (0x110, 0x111, 0x112, 0x113, 0x114, 0x117, 0x118, etc.)
            if ev_type == EventType::KEY {
                let hex_code_lower = format!("0x{:03x}", code);
                let hex_code_upper = format!("0x{:03X}", code);
                let dec_code = code.to_string();
                let is_press = value == 1;

                let matched_action = active_profile
                    .get_action(&hex_code_lower)
                    .or_else(|| active_profile.get_action(&hex_code_upper))
                    .or_else(|| active_profile.get_action(&dec_code));

                if let Some(action) = matched_action {
                    info!(
                        "Remapping KEY event on {}: code={hex_code_lower} val={value} -> action={:?}",
                        source_path.display(),
                        action
                    );
                    if is_press {
                        self.execute_action(action, code, event);
                    } else if value == 0 && matches!(action, ButtonAction::PassThrough) {
                        self.passthrough_event(event);
                    }
                    continue;
                }
            }

            // 2. Relative Wheel events (SCROLL_UP, SCROLL_DOWN, SCROLL_LEFT, SCROLL_RIGHT)
            if ev_type == EventType::RELATIVE {
                let wheel_key = match code {
                    // REL_HWHEEL (0x06) or REL_HWHEEL_HI_RES (0x0c) -> Thumb wheel
                    0x06 | 0x0c => {
                        if value < 0 {
                            Some("SCROLL_LEFT")
                        } else if value > 0 {
                            Some("SCROLL_RIGHT")
                        } else {
                            None
                        }
                    }
                    // REL_WHEEL (0x08) or REL_WHEEL_HI_RES (0x0b) -> Main wheel
                    0x08 | 0x0b => {
                        if value > 0 {
                            Some("SCROLL_UP")
                        } else if value < 0 {
                            Some("SCROLL_DOWN")
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some(key) = wheel_key {
                    if let Some(action) = active_profile.get_action(key) {
                        info!(
                            "Remapping REL wheel event on {}: key={key} val={value} -> action={:?}",
                            source_path.display(),
                            action
                        );
                        self.execute_action(action, code, event);
                        continue;
                    }
                }
            }

            // Passthrough unmapped events (movement, unmapped scroll, unmapped buttons)
            self.passthrough_event(event);
        }
    }

    /// Execute a remapped action for a button press.
    fn execute_action(&mut self, action: &ButtonAction, raw_code: u16, original_ev: &InputEvent) {
        if self.config.dry_run {
            info!(
                "DRY-RUN: Executing action for raw_code=0x{:03X}: {:?}",
                raw_code, action
            );
            return;
        }

        match action {
            ButtonAction::PassThrough => {
                self.passthrough_event(original_ev);
            }
            ButtonAction::Disabled => {
                debug!("Swallowed disabled button event code=0x{:03X}", raw_code);
            }
            ButtonAction::MouseButton { button } => {
                let target_key = match button {
                    MouseButton::Left => Key::BTN_LEFT.code(),
                    MouseButton::Right => Key::BTN_RIGHT.code(),
                    MouseButton::Middle => Key::BTN_MIDDLE.code(),
                    MouseButton::Back => Key::BTN_SIDE.code(),
                    MouseButton::Forward => Key::BTN_EXTRA.code(),
                    MouseButton::Button6 => Key::BTN_FORWARD.code(),
                    MouseButton::Button7 => Key::BTN_BACK.code(),
                };
                if let Some(ref mut vdev) = self.virtual_dev {
                    let _ = vdev.emit_click(target_key);
                }
            }
            ButtonAction::Key { key } => {
                if let Some(code) = parse_key_str(key) {
                    if let Some(ref mut vdev) = self.virtual_dev {
                        let _ = vdev.emit_click(code);
                    }
                } else {
                    warn!("Unknown key name: {key}");
                }
            }
            ButtonAction::KeyCombo { keys } => {
                let codes: Vec<u16> = keys.iter().filter_map(|k| parse_key_str(k)).collect();
                if let Some(ref mut vdev) = self.virtual_dev {
                    // Press all modifiers
                    for &c in &codes {
                        let _ = vdev.emit_key_down(c);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(15));
                    // Release all
                    for &c in codes.iter().rev() {
                        let _ = vdev.emit_key_up(c);
                    }
                }
            }
            ButtonAction::Copy => {
                self.execute_action(
                    &ButtonAction::KeyCombo {
                        keys: vec!["KEY_LEFTCTRL".to_string(), "KEY_C".to_string()],
                    },
                    raw_code,
                    original_ev,
                );
            }
            ButtonAction::Paste => {
                self.execute_action(
                    &ButtonAction::KeyCombo {
                        keys: vec!["KEY_LEFTCTRL".to_string(), "KEY_V".to_string()],
                    },
                    raw_code,
                    original_ev,
                );
            }
            ButtonAction::CloseWindow => {
                self.execute_action(
                    &ButtonAction::KeyCombo {
                        keys: vec!["KEY_LEFTALT".to_string(), "KEY_F4".to_string()],
                    },
                    raw_code,
                    original_ev,
                );
            }
            ButtonAction::MediaPlayPause => {
                if let Some(ref mut vdev) = self.virtual_dev {
                    let _ = vdev.emit_click(Key::KEY_PLAYPAUSE.code());
                }
            }
            ButtonAction::MediaNext => {
                if let Some(ref mut vdev) = self.virtual_dev {
                    let _ = vdev.emit_click(Key::KEY_NEXTSONG.code());
                }
            }
            ButtonAction::MediaPrev => {
                if let Some(ref mut vdev) = self.virtual_dev {
                    let _ = vdev.emit_click(Key::KEY_PREVIOUSSONG.code());
                }
            }
            ButtonAction::MediaMute => {
                if let Some(ref mut vdev) = self.virtual_dev {
                    let _ = vdev.emit_click(Key::KEY_MUTE.code());
                }
            }
            ButtonAction::OpenTerminal => {
                info!("Action: OpenTerminal");
                std::thread::spawn(|| {
                    let _ = std::process::Command::new("x-terminal-emulator")
                        .spawn()
                        .or_else(|_| std::process::Command::new("gnome-terminal").spawn())
                        .or_else(|_| std::process::Command::new("konsole").spawn())
                        .or_else(|_| std::process::Command::new("xfce4-terminal").spawn())
                        .or_else(|_| std::process::Command::new("alacritty").spawn())
                        .or_else(|_| std::process::Command::new("kitty").spawn())
                        .or_else(|_| std::process::Command::new("tilix").spawn());
                });
            }
            ButtonAction::RunCommand { argv } => {
                if let Err(e) = action.validate() {
                    warn!("Invalid RunCommand action: {e}");
                    return;
                }
                info!("Action: RunCommand {:?}", argv);
                let argv_clone = argv.clone();
                std::thread::spawn(move || {
                    let prog = &argv_clone[0];
                    let args = &argv_clone[1..];
                    match std::process::Command::new(prog).args(args).spawn() {
                        Ok(_) => debug!("Spawned command: {:?}", argv_clone),
                        Err(e) => error!("Failed to run command {:?}: {e}", argv_clone),
                    }
                });
            }
            ButtonAction::TypeText { text } => {
                info!("Action: TypeText ({text})");
                if let Some(ref mut vdev) = self.virtual_dev {
                    for ch in text.chars() {
                        if let Some(code) = char_to_key_code(ch) {
                            let _ = vdev.emit_click(code);
                        }
                    }
                }
            }
            ButtonAction::SwitchWorkspace { number } => {
                info!("Action: SwitchWorkspace {number}");
                let key_name = format!("KEY_{number}");
                self.execute_action(
                    &ButtonAction::KeyCombo {
                        keys: vec!["KEY_LEFTMETA".to_string(), key_name],
                    },
                    raw_code,
                    original_ev,
                );
            }
            ButtonAction::Macro { macro_id } => {
                info!("Action: Macro {macro_id}");
            }
        }
    }

    /// Forward an unmapped/passthrough event directly to the virtual device.
    fn passthrough_event(&mut self, ev: &InputEvent) {
        if let Some(ref mut vdev) = self.virtual_dev {
            let _ = vdev.emit(&[*ev]);
        }
    }
}

/// Helper function to parse key string name or hex/decimal code into evdev key code.
fn parse_key_str(name: &str) -> Option<u16> {
    if name.starts_with("0x") || name.starts_with("0X") {
        return u16::from_str_radix(&name[2..], 16).ok();
    }
    if let Ok(code) = name.parse::<u16>() {
        return Some(code);
    }

    match name.to_uppercase().as_str() {
        "KEY_A" | "A" => Some(Key::KEY_A.code()),
        "KEY_B" | "B" => Some(Key::KEY_B.code()),
        "KEY_C" | "C" => Some(Key::KEY_C.code()),
        "KEY_V" | "V" => Some(Key::KEY_V.code()),
        "KEY_X" | "X" => Some(Key::KEY_X.code()),
        "KEY_Z" | "Z" => Some(Key::KEY_Z.code()),
        "KEY_LEFTCTRL" | "CTRL" => Some(Key::KEY_LEFTCTRL.code()),
        "KEY_LEFTSHIFT" | "SHIFT" => Some(Key::KEY_LEFTSHIFT.code()),
        "KEY_LEFTALT" | "ALT" => Some(Key::KEY_LEFTALT.code()),
        "KEY_LEFTMETA" | "SUPER" | "META" => Some(Key::KEY_LEFTMETA.code()),
        "KEY_ENTER" | "ENTER" => Some(Key::KEY_ENTER.code()),
        "KEY_ESC" | "ESC" => Some(Key::KEY_ESC.code()),
        "KEY_TAB" | "TAB" => Some(Key::KEY_TAB.code()),
        "KEY_SPACE" | "SPACE" => Some(Key::KEY_SPACE.code()),
        "KEY_F4" => Some(Key::KEY_F4.code()),
        "KEY_1" => Some(Key::KEY_1.code()),
        "KEY_2" => Some(Key::KEY_2.code()),
        "KEY_3" => Some(Key::KEY_3.code()),
        "KEY_4" => Some(Key::KEY_4.code()),
        _ => None,
    }
}

/// Convert ASCII char to evdev key code.
fn char_to_key_code(c: char) -> Option<u16> {
    match c {
        'a'..='z' => Some(Key::KEY_A.code() + (c as u16 - 'a' as u16)),
        'A'..='Z' => Some(Key::KEY_A.code() + (c as u16 - 'A' as u16)),
        '1'..='9' => Some(Key::KEY_1.code() + (c as u16 - '1' as u16)),
        '0' => Some(Key::KEY_0.code()),
        ' ' => Some(Key::KEY_SPACE.code()),
        '\n' => Some(Key::KEY_ENTER.code()),
        _ => None,
    }
}

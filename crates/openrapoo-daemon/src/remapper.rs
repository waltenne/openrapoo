//! Event interceptor and remapping engine for OpenRapoo daemon.

use crate::virtual_device::OpenRapooVirtualDevice;
use anyhow::{Context, Result};
use evdev::{Device, EventType, InputEvent, Key};
use openrapoo_core::{
    config::{ButtonAction, MouseButton, ProfileStore},
    device::detect_rapoo_devices,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Remapping engine configuration.
pub struct RemapperConfig {
    pub device_path: Option<PathBuf>,
    pub config_path: Option<PathBuf>,
    pub dry_run: bool,
}

/// Main remapping service instance.
pub struct Remapper {
    config: RemapperConfig,
    profile_store: ProfileStore,
    virtual_dev: Option<OpenRapooVirtualDevice>,
    shutdown_signal: Arc<AtomicBool>,
}

impl Remapper {
    /// Create a new Remapper instance.
    pub fn new(config: RemapperConfig, shutdown_signal: Arc<AtomicBool>) -> Result<Self> {
        let config_file = config
            .config_path
            .clone()
            .unwrap_or_else(ProfileStore::default_config_path);

        let profile_store = ProfileStore::load_from_file(&config_file)
            .unwrap_or_else(|e| {
                warn!("Could not load config file: {e}. Using defaults.");
                ProfileStore::default()
            });

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

    /// Run the main remapping event loop.
    pub async fn run(&mut self) -> Result<()> {
        let evdev_paths = self.resolve_device_paths()?;

        if evdev_paths.is_empty() {
            anyhow::bail!("No accessible Rapoo evdev device nodes found to remap.");
        }

        info!("Starting remapping engine on {} node(s)", evdev_paths.len());

        let mut devices = Vec::new();
        for path in &evdev_paths {
            match Device::open(path) {
                Ok(mut dev) => {
                    let name = dev.name().unwrap_or("Unknown").to_string();
                    if !self.config.dry_run {
                        if let Err(e) = dev.grab() {
                            warn!("Could not grab device {}: {e}. Events may leak to OS.", path.display());
                        } else {
                            info!("Grabbed exclusive access on {} ({name})", path.display());
                        }
                    } else {
                        info!("DRY-RUN: Inspecting {} ({name}) without grab", path.display());
                    }
                    devices.push((path.clone(), dev));
                }
                Err(e) => {
                    warn!("Failed to open evdev node {}: {e}", path.display());
                }
            }
        }

        if devices.is_empty() {
            anyhow::bail!("Could not open any Rapoo input device nodes.");
        }

        // Event processing loop
        while !self.shutdown_signal.load(Ordering::Relaxed) {
            let mut processed_any = false;

            for (path, dev) in &mut devices {
                let events_res = tokio::task::block_in_place(|| {
                    dev.fetch_events().map(|e| e.collect::<Vec<_>>())
                });

                if let Ok(events) = events_res {
                    if !events.is_empty() {
                        processed_any = true;
                        self.process_events(path, &events);
                    }
                }
            }

            if !processed_any {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        }

        // Cleanup: ungrab devices gracefully
        for (path, mut dev) in devices {
            if !self.config.dry_run {
                let _ = dev.ungrab();
                info!("Released grab on {}", path.display());
            }
        }

        info!("Remapping engine stopped cleanly.");
        Ok(())
    }

    /// Resolve target evdev device paths.
    fn resolve_device_paths(&self) -> Result<Vec<PathBuf>> {
        if let Some(ref path) = self.config.device_path {
            return Ok(vec![path.clone()]);
        }

        let detected = detect_rapoo_devices()?;
        let mut paths = Vec::new();
        for d in detected {
            if let Some(p) = d.evdev_path {
                if !paths.contains(&p) {
                    paths.push(p);
                }
            }
        }

        Ok(paths)
    }

    /// Process a batch of raw evdev events from a physical device.
    fn process_events(&mut self, source_path: &PathBuf, events: &[InputEvent]) {
        let active_profile = self.profile_store.active_profile().clone();

        for event in events {
            let ev_type = event.event_type();
            let code = event.code();
            let value = event.value();

            if ev_type == EventType::KEY {
                let hex_code = format!("0x{:03X}", code);
                let is_press = value == 1;

                if let Some(action) = active_profile.get_action(&hex_code) {
                    debug!("Remapping event on {}: code={hex_code} val={value} -> action={:?}", source_path.display(), action);
                    if is_press {
                        self.execute_action(action, code, event);
                    } else if value == 0 && matches!(action, ButtonAction::PassThrough) {
                        self.passthrough_event(event);
                    }
                    continue;
                }
            }

            // Passthrough unmapped events (movement, scroll, unmapped buttons)
            self.passthrough_event(event);
        }
    }

    /// Execute a remapped action for a button press.
    fn execute_action(&mut self, action: &ButtonAction, raw_code: u16, original_ev: &InputEvent) {
        if self.config.dry_run {
            info!("DRY-RUN: Executing action for raw_code=0x{:03X}: {:?}", raw_code, action);
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
                    let _ = std::process::Command::new("x-terminal-emulator").spawn()
                        .or_else(|_| std::process::Command::new("gnome-terminal").spawn())
                        .or_else(|_| std::process::Command::new("konsole").spawn())
                        .or_else(|_| std::process::Command::new("xfce4-terminal").spawn());
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

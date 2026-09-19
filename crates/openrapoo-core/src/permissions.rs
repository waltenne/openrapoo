//! User group membership and udev permissions manager for OpenRapoo.

use crate::error::OpenRapooError;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Represents the state of the user's membership in the `input` group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupMembershipState {
    /// The process currently has active supplementary GID for the `input` group.
    Active,
    /// The user is listed in `/etc/group` for `input`, but the active process has not inherited the GID.
    /// User MUST log out and log back in to activate privileges.
    PendingRelogin,
    /// The user is not a member of the `input` group.
    NotMember,
}

impl GroupMembershipState {
    pub fn is_active(&self) -> bool {
        matches!(self, GroupMembershipState::Active)
    }

    pub fn display_message_pt(&self) -> &'static str {
        match self {
            GroupMembershipState::Active => "Membro ativo ✓",
            GroupMembershipState::PendingRelogin => {
                "Membro no /etc/group (Requer logout e login novamente para ativar) ⚠️"
            }
            GroupMembershipState::NotMember => "Não membro ✗ (Rode: sudo usermod -aG input $USER)",
        }
    }

    pub fn display_message_en(&self) -> &'static str {
        match self {
            GroupMembershipState::Active => "Active member ✓",
            GroupMembershipState::PendingRelogin => {
                "Member in /etc/group (Log out and back in to activate) ⚠️"
            }
            GroupMembershipState::NotMember => "Not a member ✗ (Run: sudo usermod -aG input $USER)",
        }
    }
}

/// Content of the official OpenRapoo udev rules file.
pub const OPENRAPOO_UDEV_RULES_CONTENT: &str = r#"# Rapoo MT760 Pro (Receptor 2.4 GHz Wireless / ITON Corp. VID: 0x24AE, PID: 0x186A)
# Allows read/write access for members of the 'input' group and desktop users via uaccess
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="24[aA][eE]", ATTRS{idProduct}=="18[6bB][aA]", TAG+="uaccess", GROUP="input", MODE="0660"
SUBSYSTEM=="input", ATTRS{idVendor}=="24[aA][eE]", ATTRS{idProduct}=="18[6bB][aA]", TAG+="uaccess", GROUP="input", MODE="0660"
KERNEL=="uinput", GROUP="input", MODE="0660"
"#;

/// Check the status of the user's membership in the `input` group.
pub fn check_input_group_status() -> GroupMembershipState {
    let target_user = get_target_username();
    let etc_group_content = fs::read_to_string("/etc/group").unwrap_or_default();
    let proc_status_content = fs::read_to_string("/proc/self/status").unwrap_or_default();

    check_group_status_internal(&target_user, &etc_group_content, &proc_status_content)
}

/// Core logic for group status evaluation (decoupled for unit testing).
pub fn check_group_status_internal(
    username: &str,
    etc_group_content: &str,
    proc_status_content: &str,
) -> GroupMembershipState {
    let (input_gid, listed_in_etc) = parse_etc_group_for_user(etc_group_content, username);
    let active_gids = parse_proc_status_gids(proc_status_content);

    // Also check nix getgroups() if available
    let nix_gids: Vec<u32> = nix::unistd::getgroups()
        .unwrap_or_default()
        .into_iter()
        .map(|g| g.as_raw())
        .collect();

    let has_active_gid = if let Some(gid) = input_gid {
        active_gids.contains(&gid) || nix_gids.contains(&gid)
    } else {
        false
    };

    if has_active_gid {
        GroupMembershipState::Active
    } else if listed_in_etc {
        GroupMembershipState::PendingRelogin
    } else {
        GroupMembershipState::NotMember
    }
}

/// Parse `/etc/group` content to find the `input` group GID and whether `username` is listed in it.
pub fn parse_etc_group_for_user(etc_group_content: &str, username: &str) -> (Option<u32>, bool) {
    let mut input_gid = None;
    let mut listed_in_etc = false;

    for line in etc_group_content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 && parts[0] == "input" {
            if let Ok(gid) = parts[2].parse::<u32>() {
                input_gid = Some(gid);
            }
            if parts.len() >= 4 {
                let members: Vec<&str> = parts[3].split(',').map(|s| s.trim()).collect();
                if members.contains(&username) {
                    listed_in_etc = true;
                }
            }
        }
    }

    (input_gid, listed_in_etc)
}

/// Parse `/proc/self/status` content for active supplementary `Groups:`.
pub fn parse_proc_status_gids(status_content: &str) -> Vec<u32> {
    for line in status_content.lines() {
        if line.starts_with("Groups:") {
            let parts = line.trim_start_matches("Groups:").trim();
            return parts
                .split_whitespace()
                .filter_map(|s| s.parse::<u32>().ok())
                .collect();
        }
    }
    Vec::new()
}

/// Determine the target username (respecting `SUDO_USER` if executed with `sudo`).
pub fn get_target_username() -> String {
    if let Ok(sudo_user) = env::var("SUDO_USER") {
        if !sudo_user.trim().is_empty() {
            return sudo_user.trim().to_string();
        }
    }

    env::var("USER")
        .or_else(|_| env::var("LOGNAME"))
        .unwrap_or_else(|_| {
            let uid = nix::unistd::getuid();
            uid.to_string()
        })
}

/// Install OpenRapoo udev rules into `/etc/udev/rules.d/99-openrapoo.rules`.
///
/// Requires root privileges.
pub fn install_udev_rules(custom_target_dir: Option<&Path>) -> Result<PathBuf, OpenRapooError> {
    let is_root = nix::unistd::geteuid().is_root();

    let target_dir = custom_target_dir.unwrap_or_else(|| Path::new("/etc/udev/rules.d"));
    let target_file = target_dir.join("99-openrapoo.rules");

    if !is_root && custom_target_dir.is_none() {
        return Err(OpenRapooError::PermissionDenied(
            "Erro: '--install-udev' requer privilégios de administrador (root). Execute com 'sudo openrapoo-gui --install-udev'.".to_string(),
        ));
    }

    if let Some(parent) = target_file.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&target_file, OPENRAPOO_UDEV_RULES_CONTENT)?;
    info!(
        "Regras udev instaladas com sucesso em {}",
        target_file.display()
    );

    // Only attempt udevadm if writing to standard system location
    if custom_target_dir.is_none() {
        let reload_res = std::process::Command::new("udevadm")
            .args(["control", "--reload-rules"])
            .output();
        if let Err(e) = reload_res {
            warn!("Aviso: não foi possível executar udevadm control --reload-rules: {e}");
        }

        let trigger_res = std::process::Command::new("udevadm")
            .arg("trigger")
            .output();
        if let Err(e) = trigger_res {
            warn!("Aviso: não foi possível executar udevadm trigger: {e}");
        }
    }

    Ok(target_file)
}

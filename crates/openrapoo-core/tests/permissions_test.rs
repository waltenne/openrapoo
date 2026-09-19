//! Unit tests for group permissions detection, config path resolution, and udev installation logic.

use openrapoo_core::{
    config::ProfileStore,
    permissions::{
        check_group_status_internal, install_udev_rules, parse_proc_status_gids,
        GroupMembershipState,
    },
};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_etc_group_parsing_active_and_pending() {
    let fake_etc_group = r#"
root:x:0:
daemon:x:1:
input:x:104:user1,user2
wheel:x:999:user1
"#;

    let fake_proc_status_active = r#"
Name:	openrapoo-gui
State:	S (sleeping)
Groups:	1000 999 104
"#;

    let fake_proc_status_pending = r#"
Name:	openrapoo-gui
State:	S (sleeping)
Groups:	1000 999
"#;

    // Test active member
    let status_active =
        check_group_status_internal("user1", fake_etc_group, fake_proc_status_active);
    assert_eq!(status_active, GroupMembershipState::Active);
    assert!(status_active.is_active());

    // Test pending relogin member (user in /etc/group but GID 104 not in active Groups:)
    let status_pending =
        check_group_status_internal("user1", fake_etc_group, fake_proc_status_pending);
    assert_eq!(status_pending, GroupMembershipState::PendingRelogin);
    assert!(!status_pending.is_active());

    // Test non-member
    let status_non_member =
        check_group_status_internal("user3", fake_etc_group, fake_proc_status_pending);
    assert_eq!(status_non_member, GroupMembershipState::NotMember);
    assert!(!status_non_member.is_active());
}

#[test]
fn test_proc_status_gids_parsing() {
    let proc_status = r#"
Name:	test
Pid:	1234
Groups:	10 20 104 1000
"#;
    let gids = parse_proc_status_gids(proc_status);
    assert_eq!(gids, vec![10, 20, 104, 1000]);
}

#[test]
fn test_config_path_resolution_never_root_for_normal_users() {
    // 1. Normal user with HOME set
    let path_normal = ProfileStore::resolve_config_path_internal(None, None, Some("/home/user1"));
    assert_eq!(
        path_normal,
        PathBuf::from("/home/user1/.config/openrapoo/profiles.json")
    );
    assert!(!path_normal.to_string_lossy().contains("/root/"));

    // 2. Custom XDG_CONFIG_HOME set
    let path_xdg = ProfileStore::resolve_config_path_internal(
        None,
        Some("/custom/config"),
        Some("/home/user1"),
    );
    assert_eq!(
        path_xdg,
        PathBuf::from("/custom/config/openrapoo/profiles.json")
    );
    assert!(!path_xdg.to_string_lossy().contains("/root/"));

    // 3. User ran under sudo (SUDO_USER set) — should resolve to SUDO_USER's home
    let test_user = std::env::var("USER").unwrap_or_else(|_| "testuser".to_string());
    let path_sudo =
        ProfileStore::resolve_config_path_internal(Some(&test_user), None, Some("/root"));
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from(format!("/home/{test_user}")));
    assert_eq!(
        path_sudo,
        home_dir.join(".config/openrapoo/profiles.json")
    );
    assert!(!path_sudo.to_string_lossy().contains("/root/"));
}

#[test]
fn test_udev_rule_installation_custom_dir() {
    let temp_dir = std::env::temp_dir().join("openrapoo_udev_test");
    let res = install_udev_rules(Some(&temp_dir));

    assert!(res.is_ok());
    let installed_file = res.unwrap();
    assert!(installed_file.exists());
    assert_eq!(installed_file, temp_dir.join("99-openrapoo.rules"));

    let content = fs::read_to_string(&installed_file).unwrap();
    assert!(content.contains("ATTRS{idVendor}==\"24[aA][eE]\""));
    assert!(content.contains("ATTRS{idProduct}==\"18[6bB][aA]\""));
    assert!(content.contains("TAG+=\"uaccess\""));
    assert!(content.contains("GROUP=\"input\""));

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_group_membership_display_messages() {
    let active = GroupMembershipState::Active;
    let pending = GroupMembershipState::PendingRelogin;
    let not_member = GroupMembershipState::NotMember;

    assert!(active.display_message_pt().contains("Membro ativo"));
    assert!(pending.display_message_pt().contains("logout e login"));
    assert!(not_member.display_message_pt().contains("Não membro"));
}

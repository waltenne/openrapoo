use std::path::PathBuf;
use tracing::warn;

/// Resolve the absolute path of `openrapoo-mt760-pro.svg`.
#[allow(dead_code)]
pub fn get_mt760_pro_svg_path() -> Option<PathBuf> {
    const FILENAME: &str = "openrapoo-mt760-pro.svg";

    // 1. Current working directory
    let cwd_path = PathBuf::from(FILENAME);
    if cwd_path.exists() {
        return Some(cwd_path.canonicalize().unwrap_or(cwd_path));
    }

    // 2. Cargo manifest directory (development mode)
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = PathBuf::from(&manifest_dir).join(FILENAME);
        if manifest_path.exists() {
            return Some(manifest_path);
        }
        let root_path = PathBuf::from(&manifest_dir)
            .join("..")
            .join("..")
            .join(FILENAME);
        if root_path.exists() {
            return Some(root_path.canonicalize().unwrap_or(root_path));
        }
    }

    // 3. Executable directory (release / installed mode)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let relative_to_exe = exe_dir.join(FILENAME);
            if relative_to_exe.exists() {
                return Some(relative_to_exe);
            }
            let assets_dir = exe_dir.join("assets").join(FILENAME);
            if assets_dir.exists() {
                return Some(assets_dir);
            }
        }
    }

    warn!("Could not locate asset file {FILENAME}");
    None
}

/// Resolve the absolute path of `openrapoo-e9050l.svg` keyboard asset.
#[allow(dead_code)]
pub fn get_e9050l_svg_path() -> Option<PathBuf> {
    const FILENAME: &str = "openrapoo-e9050l.svg";

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_assets = PathBuf::from(&manifest_dir).join("assets").join(FILENAME);
        if manifest_assets.exists() {
            return Some(manifest_assets.canonicalize().unwrap_or(manifest_assets));
        }
        let manifest_path = PathBuf::from(&manifest_dir).join(FILENAME);
        if manifest_path.exists() {
            return Some(manifest_path);
        }
    }

    let cwd_assets = PathBuf::from("assets").join(FILENAME);
    if cwd_assets.exists() {
        return Some(cwd_assets.canonicalize().unwrap_or(cwd_assets));
    }
    let cwd_path = PathBuf::from(FILENAME);
    if cwd_path.exists() {
        return Some(cwd_path.canonicalize().unwrap_or(cwd_path));
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let assets_dir = exe_dir.join("assets").join(FILENAME);
            if assets_dir.exists() {
                return Some(assets_dir);
            }
            let relative_to_exe = exe_dir.join(FILENAME);
            if relative_to_exe.exists() {
                return Some(relative_to_exe);
            }
        }
    }

    warn!("Could not locate asset file {FILENAME}");
    None
}

/// Resolve the path of `mt760-pro.png` asset across development, release, Flatpak and AppImage.
pub fn get_mt760_pro_png_path() -> Option<PathBuf> {
    const ASSET_NAME: &str = "mt760-pro.png";

    // 1. Cargo manifest directory & assets subfolder (development mode)
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_assets = PathBuf::from(&manifest_dir).join("assets").join(ASSET_NAME);
        if manifest_assets.exists() {
            return Some(manifest_assets.canonicalize().unwrap_or(manifest_assets));
        }
        let manifest_file = PathBuf::from(&manifest_dir).join(ASSET_NAME);
        if manifest_file.exists() {
            return Some(manifest_file.canonicalize().unwrap_or(manifest_file));
        }
        let root_assets = PathBuf::from(&manifest_dir)
            .join("..")
            .join("..")
            .join("crates")
            .join("openrapoo-gui")
            .join("assets")
            .join(ASSET_NAME);
        if root_assets.exists() {
            return Some(root_assets.canonicalize().unwrap_or(root_assets));
        }
    }

    // 2. Current working directory / relative paths
    let cwd_assets = PathBuf::from("assets").join(ASSET_NAME);
    if cwd_assets.exists() {
        return Some(cwd_assets.canonicalize().unwrap_or(cwd_assets));
    }
    let cwd_file = PathBuf::from(ASSET_NAME);
    if cwd_file.exists() {
        return Some(cwd_file.canonicalize().unwrap_or(cwd_file));
    }

    // 3. Executable relative path / installed AppImage / Flatpak share
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let relative_to_exe = exe_dir.join("assets").join(ASSET_NAME);
            if relative_to_exe.exists() {
                return Some(relative_to_exe);
            }
            let same_dir = exe_dir.join(ASSET_NAME);
            if same_dir.exists() {
                return Some(same_dir);
            }
        }
    }

    // 4. User local data share directory (~/.local/share/openrapoo/assets)
    if let Ok(home) = std::env::var("HOME") {
        let user_share = PathBuf::from(&home)
            .join(".local")
            .join("share")
            .join("openrapoo")
            .join("assets")
            .join(ASSET_NAME);
        if user_share.exists() {
            return Some(user_share);
        }
    }

    // 5. Hardcoded workspace fallback directory
    let workspace_fallback = PathBuf::from("crates/openrapoo-gui/assets").join(ASSET_NAME);
    if workspace_fallback.exists() {
        return Some(workspace_fallback);
    }

    // 6. System wide Linux share directory (Flatpak / AppImage / system package)
    let system_share = PathBuf::from("/usr/share/openrapoo/assets").join(ASSET_NAME);
    if system_share.exists() {
        return Some(system_share);
    }

    warn!("Could not locate asset file {ASSET_NAME}");
    None
}

/// Resolve the path of `e9050l.png` / `E9050L.png` keyboard image asset.
pub fn get_e9050l_png_path() -> Option<PathBuf> {
    for name in ["e9050l.png", "E9050L.png"] {
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let p = PathBuf::from(&manifest_dir).join("assets").join(name);
            if p.exists() {
                return Some(p.canonicalize().unwrap_or(p));
            }
            let p_file = PathBuf::from(&manifest_dir).join(name);
            if p_file.exists() {
                return Some(p_file.canonicalize().unwrap_or(p_file));
            }
            let root_file = PathBuf::from(&manifest_dir)
                .join("..")
                .join("..")
                .join(name);
            if root_file.exists() {
                return Some(root_file.canonicalize().unwrap_or(root_file));
            }
        }
        let cwd_assets = PathBuf::from("assets").join(name);
        if cwd_assets.exists() {
            return Some(cwd_assets.canonicalize().unwrap_or(cwd_assets));
        }
        let cwd_file = PathBuf::from(name);
        if cwd_file.exists() {
            return Some(cwd_file.canonicalize().unwrap_or(cwd_file));
        }
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let p = exe_dir.join("assets").join(name);
                if p.exists() {
                    return Some(p);
                }
                let same_dir = exe_dir.join(name);
                if same_dir.exists() {
                    return Some(same_dir);
                }
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let user_share = PathBuf::from(&home)
                .join(".local")
                .join("share")
                .join("openrapoo")
                .join("assets")
                .join(name);
            if user_share.exists() {
                return Some(user_share);
            }
        }
        let workspace_fallback = PathBuf::from("crates/openrapoo-gui/assets").join(name);
        if workspace_fallback.exists() {
            return Some(workspace_fallback);
        }
    }

    warn!("Could not locate keyboard asset file e9050l.png / E9050L.png");
    None
}

/// Resolve the path of `icon.png` application icon asset.
pub fn get_app_icon_path() -> Option<PathBuf> {
    const ASSET_NAME: &str = "icon.png";

    let path = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_assets = PathBuf::from(&manifest_dir).join("assets").join(ASSET_NAME);
        if manifest_assets.exists() {
            Some(manifest_assets.canonicalize().unwrap_or(manifest_assets))
        } else {
            let manifest_file = PathBuf::from(&manifest_dir).join(ASSET_NAME);
            if manifest_file.exists() {
                Some(manifest_file.canonicalize().unwrap_or(manifest_file))
            } else {
                let root_icon = PathBuf::from(&manifest_dir)
                    .join("..")
                    .join("..")
                    .join(ASSET_NAME);
                if root_icon.exists() {
                    Some(root_icon.canonicalize().unwrap_or(root_icon))
                } else {
                    None
                }
            }
        }
    } else {
        None
    };

    if let Some(p) = path {
        tracing::debug!("Resolved app icon asset at {:?}", p);
        return Some(p);
    }

    let cwd_assets = PathBuf::from("assets").join(ASSET_NAME);
    if cwd_assets.exists() {
        let p = cwd_assets.canonicalize().unwrap_or(cwd_assets);
        tracing::debug!("Resolved app icon asset at {:?}", p);
        return Some(p);
    }
    let cwd_file = PathBuf::from(ASSET_NAME);
    if cwd_file.exists() {
        let p = cwd_file.canonicalize().unwrap_or(cwd_file);
        tracing::debug!("Resolved app icon asset at {:?}", p);
        return Some(p);
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let relative_to_exe = exe_dir.join("assets").join(ASSET_NAME);
            if relative_to_exe.exists() {
                tracing::debug!("Resolved app icon asset at {:?}", relative_to_exe);
                return Some(relative_to_exe);
            }
            let same_dir = exe_dir.join(ASSET_NAME);
            if same_dir.exists() {
                tracing::debug!("Resolved app icon asset at {:?}", same_dir);
                return Some(same_dir);
            }
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let user_share = PathBuf::from(&home)
            .join(".local")
            .join("share")
            .join("openrapoo")
            .join("assets")
            .join(ASSET_NAME);
        if user_share.exists() {
            tracing::debug!("Resolved app icon asset at {:?}", user_share);
            return Some(user_share);
        }
        let user_app_icon = PathBuf::from(&home)
            .join(".local")
            .join("share")
            .join("icons")
            .join("hicolor")
            .join("256x256")
            .join("apps")
            .join("openrapoo-gui.png");
        if user_app_icon.exists() {
            tracing::debug!("Resolved app icon asset at {:?}", user_app_icon);
            return Some(user_app_icon);
        }
    }

    let workspace_fallback = PathBuf::from("crates/openrapoo-gui/assets").join(ASSET_NAME);
    if workspace_fallback.exists() {
        tracing::debug!("Resolved app icon asset at {:?}", workspace_fallback);
        return Some(workspace_fallback);
    }

    let system_share = PathBuf::from("/usr/share/pixmaps").join(ASSET_NAME);
    if system_share.exists() {
        tracing::debug!("Resolved app icon asset at {:?}", system_share);
        return Some(system_share);
    }
    let system_share_icons = PathBuf::from("/usr/share/icons/hicolor/256x256/apps")
        .join("io.github.openrapoo.OpenRapoo.png");
    if system_share_icons.exists() {
        tracing::debug!("Resolved app icon asset at {:?}", system_share_icons);
        return Some(system_share_icons);
    }

    warn!("Could not locate asset file {ASSET_NAME}");
    None
}

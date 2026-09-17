#[allow(dead_code)]
use crate::views::{
    diag_view::DiagViewState,
    home_view::HomeViewState,
    logs_view::LogsViewState,
    permissions_view::PermissionsCheckStatus,
    profiles_view::ProfilesViewState,
};

#[allow(dead_code)]

pub struct AppWindowController {
    pub home_state: HomeViewState,
    pub profiles_state: ProfilesViewState,
    pub diag_state: DiagViewState,
    pub permissions_status: PermissionsCheckStatus,
    pub logs_state: LogsViewState,
}

impl Default for AppWindowController {
    fn default() -> Self {
        AppWindowController {
            home_state: HomeViewState::default(),
            profiles_state: ProfilesViewState::default(),
            diag_state: DiagViewState::default(),
            permissions_status: PermissionsCheckStatus::check(),
            logs_state: LogsViewState::default(),
        }
    }
}

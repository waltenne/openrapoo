//! Service abstractions connecting GUI views to openrapoo_core, filesystem stores, and daemon IPC.

pub mod battery_service;
pub mod daemon_client;
pub mod device_service;
pub mod profile_service;

pub use battery_service::BatteryService;
pub use daemon_client::DaemonClient;
pub use device_service::SystemDeviceService;
pub use profile_service::ProfileService;

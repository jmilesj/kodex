#![cfg(not(debug_assertions))]

use crate::legacy_core::config::Config;

pub fn get_upgrade_version(_config: &Config) -> Option<String> {
    None
}

/// Returns the latest version to show in a popup, if it should be shown.
pub fn get_upgrade_version_for_popup(_config: &Config) -> Option<String> {
    None
}

/// Persist a dismissal for the current latest version so we don't show
/// the update popup again for this version.
pub async fn dismiss_version(_config: &Config, _version: &str) -> anyhow::Result<()> {
    Ok(())
}

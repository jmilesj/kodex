use anyhow::Result;
#[cfg(not(unix))]
use anyhow::bail;

#[cfg(unix)]
use crate::RestartMode;
#[cfg(unix)]
use crate::UpdaterRefreshMode;
#[cfg(unix)]
use crate::managed_install::ExecutableIdentity;

#[cfg(unix)]
pub(crate) async fn run() -> Result<()> {
    Ok(())
}

#[cfg(not(unix))]
pub(crate) async fn run() -> Result<()> {
    bail!("pid-managed updater loop is unsupported on this platform")
}

#[cfg(unix)]
fn update_modes_for_identities(
    _running_updater_identity: &ExecutableIdentity,
    _managed_identity: &ExecutableIdentity,
) -> (RestartMode, UpdaterRefreshMode) {
    (RestartMode::IfVersionChanged, UpdaterRefreshMode::None)
}

#[cfg(unix)]
pub(crate) fn reexec_managed_updater(_managed_codex_bin: &std::path::Path) -> Result<()> {
    Ok(())
}

#[cfg(all(test, unix))]
#[path = "update_loop_tests.rs"]
mod tests;

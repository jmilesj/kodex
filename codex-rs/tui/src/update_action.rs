#[cfg(any(not(debug_assertions), test))]
use codex_install_context::InstallContext;
#[cfg(any(not(debug_assertions), test))]
use codex_install_context::InstallMethod;
#[cfg(any(not(debug_assertions), test))]
use codex_install_context::StandalonePlatform;

const INSTALL_SCRIPT_COMMAND: &str =
    "curl -fsSL https://github.com/jmilesj/kodex/releases/latest/download/install.sh | sh";

/// Update action the CLI should perform after the TUI exits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateAction {
    /// Update via the disabled `kodex update` command.
    NpmGlobalLatest,
    /// Update via the disabled `kodex update` command.
    BunGlobalLatest,
    /// Update via the disabled `kodex update` command.
    BrewUpgrade,
    /// Update via the disabled `kodex update` command.
    StandaloneUnix,
}

impl UpdateAction {
    #[cfg(any(not(debug_assertions), test))]
    pub(crate) fn from_install_context(context: &InstallContext) -> Option<Self> {
        match &context.method {
            InstallMethod::Npm => Some(UpdateAction::NpmGlobalLatest),
            InstallMethod::Bun => Some(UpdateAction::BunGlobalLatest),
            InstallMethod::Brew => Some(UpdateAction::BrewUpgrade),
            InstallMethod::Standalone {
                platform: StandalonePlatform::Unix,
                ..
            } => Some(UpdateAction::StandaloneUnix),
            InstallMethod::Standalone {
                platform: StandalonePlatform::Windows,
                ..
            } => None,
            InstallMethod::Other => None,
        }
    }

    /// Returns the list of command-line arguments for invoking the update.
    pub fn command_args(self) -> (&'static str, &'static [&'static str]) {
        match self {
            UpdateAction::NpmGlobalLatest => ("npm", &["install", "-g", "kodex"]),
            UpdateAction::BunGlobalLatest => ("bun", &["install", "-g", "kodex"]),
            UpdateAction::BrewUpgrade => ("brew", &["upgrade", "--cask", "kodex"]),
            UpdateAction::StandaloneUnix => ("sh", &["-c", INSTALL_SCRIPT_COMMAND]),
        }
    }

    /// Returns string representation of the command-line arguments for invoking the update.
    pub fn command_str(self) -> String {
        let (command, args) = self.command_args();
        shlex::try_join(std::iter::once(command).chain(args.iter().copied()))
            .unwrap_or_else(|_| format!("{command} {}", args.join(" ")))
    }
}

#[cfg(not(debug_assertions))]
pub fn get_update_action() -> Option<UpdateAction> {
    let _ = InstallContext::current();
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use pretty_assertions::assert_eq;

    #[test]
    fn maps_install_context_to_update_action() {
        let native_release_dir =
            AbsolutePathBuf::from_absolute_path(std::env::temp_dir().join("native-release"))
                .expect("temp dir path should be absolute");

        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Other,
                package_layout: None,
            }),
            None
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Npm,
                package_layout: None,
            }),
            Some(UpdateAction::NpmGlobalLatest)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Bun,
                package_layout: None,
            }),
            Some(UpdateAction::BunGlobalLatest)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Brew,
                package_layout: None,
            }),
            Some(UpdateAction::BrewUpgrade)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Standalone {
                    platform: StandalonePlatform::Unix,
                    release_dir: native_release_dir.clone(),
                    resources_dir: Some(native_release_dir.join("codex-resources")),
                },
                package_layout: None,
            }),
            Some(UpdateAction::StandaloneUnix)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext {
                method: InstallMethod::Standalone {
                    platform: StandalonePlatform::Windows,
                    release_dir: native_release_dir.clone(),
                    resources_dir: Some(native_release_dir.join("codex-resources")),
                },
                package_layout: None,
            }),
            None
        );
    }

    #[test]
    fn update_commands_delegate_to_disabled_update_subcommand() {
        assert_eq!(
            UpdateAction::StandaloneUnix.command_args(),
            ("sh", &["-c", INSTALL_SCRIPT_COMMAND][..],)
        );
    }
}

pub const KODEX_CLI_VERSION: &str =
    resolve_version(option_env!("KODEX_CLI_VERSION"), env!("CARGO_PKG_VERSION"));

pub const fn resolve_version<'a>(
    release_version: Option<&'a str>,
    cargo_version: &'a str,
) -> &'a str {
    match release_version {
        Some(version) => version,
        None => cargo_version,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_version_override_wins() {
        assert_eq!(
            resolve_version(Some("0.133.0.1779638524"), "0.0.0"),
            "0.133.0.1779638524"
        );
    }

    #[test]
    fn cargo_package_version_is_fallback() {
        assert_eq!(resolve_version(None, "0.0.0"), "0.0.0");
    }
}

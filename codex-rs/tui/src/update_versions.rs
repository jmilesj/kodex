pub(crate) fn is_newer(latest: &str, current: &str) -> Option<bool> {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => Some(l > c),
        _ => None,
    }
}

pub(crate) fn extract_version_from_latest_tag(latest_tag_name: &str) -> anyhow::Result<String> {
    let mut version = latest_tag_name;
    let mut stripped_any_prefix = false;

    loop {
        if let Some(rest) = version.strip_prefix("kodex-v") {
            version = rest;
            stripped_any_prefix = true;
            continue;
        }
        if let Some(rest) = version.strip_prefix("rust-v") {
            version = rest;
            stripped_any_prefix = true;
            continue;
        }
        if let Some(rest) = version.strip_prefix('v') {
            version = rest;
            stripped_any_prefix = true;
            continue;
        }
        break;
    }

    if stripped_any_prefix {
        Ok(version
            .split_once('+')
            .map_or(version, |(base, _)| base)
            .to_owned())
    } else {
        Err(anyhow::anyhow!(
            "Failed to parse latest tag name '{latest_tag_name}'"
        ))
    }
}

pub(crate) fn is_source_build_version(version: &str) -> bool {
    version.trim() == "0.0.0"
}

fn parse_version(v: &str) -> Option<(u64, u64, u64)> {
    let base_version = v.trim().split_once('+').map_or(v.trim(), |(base, _)| base);
    let mut iter = base_version.split('.');
    let maj = iter.next()?.parse::<u64>().ok()?;
    let min = iter.next()?.parse::<u64>().ok()?;
    let pat = iter.next()?.parse::<u64>().ok()?;
    Some((maj, min, pat))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn extracts_version_from_latest_tag() {
        assert_eq!(
            extract_version_from_latest_tag("kodex-v1.5.0+20260524.120000")
                .expect("failed to parse version"),
            "1.5.0"
        );
        assert_eq!(
            extract_version_from_latest_tag("rust-vrust-v0.99.0-alpha.16")
                .expect("failed to parse version"),
            "0.99.0-alpha.16"
        );
    }

    #[test]
    fn latest_tag_with_v_prefix_is_accepted() {
        assert_eq!(
            extract_version_from_latest_tag("v1.5.0+20260524.120000")
                .expect("failed to parse version"),
            "1.5.0"
        );
    }

    #[test]
    fn latest_tag_without_prefix_is_invalid() {
        assert!(extract_version_from_latest_tag("release-1.5.0").is_err());
    }

    #[test]
    fn prerelease_version_is_not_considered_newer() {
        assert_eq!(is_newer("0.11.0-beta.1", "0.11.0"), None);
        assert_eq!(is_newer("1.0.0-rc.1", "1.0.0"), None);
    }

    #[test]
    fn plain_semver_comparisons_work() {
        assert_eq!(is_newer("0.11.1", "0.11.0"), Some(true));
        assert_eq!(is_newer("0.11.0", "0.11.1"), Some(false));
        assert_eq!(is_newer("1.0.0", "0.9.9"), Some(true));
        assert_eq!(is_newer("0.9.9", "1.0.0"), Some(false));
    }

    #[test]
    fn source_build_version_is_not_checked() {
        assert!(is_source_build_version("0.0.0"));
        assert!(!is_source_build_version("0.0.0+20260524.120000"));
        assert!(!is_source_build_version("0.1.0"));
    }

    #[test]
    fn whitespace_is_ignored() {
        assert_eq!(parse_version(" 1.2.3 \n"), Some((1, 2, 3)));
        assert_eq!(is_newer(" 1.2.3 ", "1.2.2"), Some(true));
    }

    #[test]
    fn build_metadata_is_ignored_for_comparison() {
        assert_eq!(is_newer("1.2.4+20260524.120000", "1.2.3"), Some(true));
        assert_eq!(is_newer("1.2.3+20260524.120000", "1.2.3"), Some(false));
    }
}

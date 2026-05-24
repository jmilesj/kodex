#!/usr/bin/env python3

from __future__ import annotations

import re
import sys
from dataclasses import dataclass


@dataclass(frozen=True)
class ReleaseMetadata:
    base_version: str
    cargo_version: str
    display_version: str
    release_tag: str


def normalize_base_version(tag: str) -> str:
    version = tag.strip()
    while True:
        if version.startswith("kodex-v"):
            version = version.removeprefix("kodex-v")
        elif version.startswith("rust-v"):
            version = version.removeprefix("rust-v")
        elif version.startswith("v"):
            version = version.removeprefix("v")
        else:
            break

    match = re.match(r"^(\d+)\.(\d+)\.(\d+)", version)
    if match is None:
        raise ValueError(f"could not parse base version from release tag: {tag!r}")

    return ".".join(match.groups())


def resolve_release_metadata(upstream_tag: str, build_number: str) -> ReleaseMetadata:
    if not re.fullmatch(r"\d+", build_number):
        raise ValueError(f"build number must be numeric: {build_number!r}")

    base_version = normalize_base_version(upstream_tag)
    display_version = f"{base_version}.{build_number}"
    return ReleaseMetadata(
        base_version=base_version,
        cargo_version=f"{base_version}+{build_number}",
        display_version=display_version,
        release_tag=f"kodex-v{display_version}",
    )


def main() -> int:
    if len(sys.argv) != 3:
        print(
            "usage: kodex_release_version.py UPSTREAM_RELEASE_TAG BUILD_NUMBER",
            file=sys.stderr,
        )
        return 2

    metadata = resolve_release_metadata(sys.argv[1], sys.argv[2])
    print(f"base_version={metadata.base_version}")
    print(f"cargo_version={metadata.cargo_version}")
    print(f"display_version={metadata.display_version}")
    print(f"version={metadata.display_version}")
    print(f"release_tag={metadata.release_tag}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

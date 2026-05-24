#!/usr/bin/env python3

from __future__ import annotations

import unittest

import kodex_release_version


class KodexReleaseVersionTest(unittest.TestCase):
    def test_resolves_upstream_tag_to_fork_versions(self) -> None:
        metadata = kodex_release_version.resolve_release_metadata(
            upstream_tag="rust-v0.133.0",
            build_number="1779638524",
        )

        self.assertEqual(
            metadata,
            kodex_release_version.ReleaseMetadata(
                base_version="0.133.0",
                cargo_version="0.133.0+1779638524",
                display_version="0.133.0.1779638524",
                release_tag="kodex-v0.133.0.1779638524",
            ),
        )

    def test_normalizes_supported_tag_prefixes_and_suffixes(self) -> None:
        cases = {
            "rust-v0.133.0": "0.133.0",
            "kodex-v0.133.0.1779638524": "0.133.0",
            "v0.133.0+20260524": "0.133.0",
            "0.133.0-beta.1": "0.133.0",
        }

        for tag, expected in cases.items():
            with self.subTest(tag=tag):
                self.assertEqual(kodex_release_version.normalize_base_version(tag), expected)


if __name__ == "__main__":
    unittest.main()

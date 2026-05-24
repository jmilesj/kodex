#!/usr/bin/env python3

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
RELEASE_WORKFLOW = ROOT / ".github" / "workflows" / "release.yml"


class ReleaseWorkflowTest(unittest.TestCase):
    def test_build_uses_release_version_env_without_rewriting_cargo_toml(self) -> None:
        workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("KODEX_CLI_VERSION: ${{ needs.metadata.outputs.version }}", workflow)
        self.assertNotIn("Stamp release version into Cargo.toml", workflow)
        self.assertNotIn("path = Path(\"Cargo.toml\")", workflow)

    def test_cache_key_is_per_commit_with_lockfile_restore_prefix(self) -> None:
        workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("${{ github.sha }}", workflow)
        self.assertIn(
            "kodex-release-${{ runner.os }}-${{ matrix.target }}-${{ hashFiles('codex-rs/Cargo.lock', 'codex-rs/Cargo.toml') }}-",
            workflow,
        )


if __name__ == "__main__":
    unittest.main()

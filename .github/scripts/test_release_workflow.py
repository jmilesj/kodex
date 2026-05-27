#!/usr/bin/env python3

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
RELEASE_WORKFLOW = ROOT / ".github" / "workflows" / "release.yml"


class ReleaseWorkflowTest(unittest.TestCase):
    def test_metadata_job_checks_out_repo_before_resolving_version(self) -> None:
        workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")
        lines = workflow.splitlines()

        metadata_start = lines.index("  metadata:")
        build_start = lines.index("  build:")
        metadata_block = "\n".join(lines[metadata_start:build_start])

        self.assertIn("uses: actions/checkout@", metadata_block)
        self.assertLess(
            metadata_block.index("uses: actions/checkout@"),
            metadata_block.index("Resolve release version"),
        )

    def test_build_job_uses_rust_cache_instead_of_manual_cache_steps(self) -> None:
        workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("uses: Swatinem/rust-cache@v2", workflow)
        self.assertIn("workspaces: |", workflow)
        self.assertIn("codex-rs -> target", workflow)
        self.assertIn("key: ${{ matrix.target }}", workflow)
        self.assertNotIn("uses: actions/cache/restore@v4", workflow)
        self.assertNotIn("uses: actions/cache/save@v4", workflow)

    def test_build_uses_release_version_env_without_rewriting_cargo_toml(self) -> None:
        workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("KODEX_CLI_VERSION: ${{ needs.metadata.outputs.version }}", workflow)
        self.assertIn('CARGO_PROFILE_RELEASE_LTO: "false"', workflow)
        self.assertIn('CARGO_PROFILE_RELEASE_OPT_LEVEL: "2"', workflow)
        self.assertNotIn("Stamp release version into Cargo.toml", workflow)
        self.assertNotIn("path = Path(\"Cargo.toml\")", workflow)

    def test_linux_release_targets_use_gnu_with_zigbuild_glibc_floor(self) -> None:
        workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("target: x86_64-unknown-linux-gnu", workflow)
        self.assertIn("build_target: x86_64-unknown-linux-gnu.2.17", workflow)
        self.assertIn("target: aarch64-unknown-linux-gnu", workflow)
        self.assertIn("build_target: aarch64-unknown-linux-gnu.2.17", workflow)
        self.assertNotIn("target: x86_64-unknown-linux-musl", workflow)
        self.assertNotIn("target: aarch64-unknown-linux-musl", workflow)

    def test_linux_release_build_uses_cargo_zigbuild(self) -> None:
        workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("cargo-zigbuild==0.22.3", workflow)
        self.assertIn("ziglang==0.16.0", workflow)
        self.assertNotIn("install-musl-build-tools.sh", workflow)
        self.assertIn("multiarch=\"$(gcc -dumpmachine)\"", workflow)
        self.assertIn("cflags=\"-isystem /usr/include -isystem /usr/include/${multiarch}\"", workflow)
        self.assertIn("CFLAGS=\"$cflags\" RUSTFLAGS=\"$rustflags\"", workflow)
        self.assertIn("cargo zigbuild --release --timings -p codex-cli --bin kodex --target \"$BUILD_TARGET\"", workflow)
        self.assertIn("cargo build --release --timings -p codex-cli --bin kodex --target \"$TARGET\"", workflow)

    def test_release_workflow_uploads_cargo_timings(self) -> None:
        workflow = RELEASE_WORKFLOW.read_text(encoding="utf-8")

        self.assertIn("name: Upload cargo timings", workflow)
        self.assertIn("if: always()", workflow)
        self.assertIn("name: cargo-timings-${{ matrix.target }}", workflow)
        self.assertIn("path: codex-rs/target/cargo-timings", workflow)


if __name__ == "__main__":
    unittest.main()

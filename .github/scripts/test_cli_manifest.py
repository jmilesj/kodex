#!/usr/bin/env python3

from __future__ import annotations

import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CLI_CARGO_TOML = ROOT / "codex-rs" / "cli" / "Cargo.toml"
TUI_CARGO_TOML = ROOT / "codex-rs" / "tui" / "Cargo.toml"
CODEX_CARGO_LOCK = ROOT / "codex-rs" / "Cargo.lock"
CLI_MAIN_RS = ROOT / "codex-rs" / "cli" / "src" / "main.rs"


class CliManifestTest(unittest.TestCase):
    def test_release_cli_does_not_depend_on_app_server_test_client(self) -> None:
        manifest = tomllib.loads(CLI_CARGO_TOML.read_text(encoding="utf-8"))
        dependencies = manifest["dependencies"]
        main_rs = CLI_MAIN_RS.read_text(encoding="utf-8")

        self.assertNotIn("codex-app-server-test-client", dependencies)
        self.assertNotIn("codex_app_server_test_client", main_rs)

    def test_release_cli_does_not_pull_native_webrtc(self) -> None:
        tui_manifest = tomllib.loads(TUI_CARGO_TOML.read_text(encoding="utf-8"))
        tui_dependencies = tui_manifest["dependencies"]
        lockfile = tomllib.loads(CODEX_CARGO_LOCK.read_text(encoding="utf-8"))
        locked_packages = {package["name"] for package in lockfile["package"]}

        self.assertNotIn("codex-realtime-webrtc", tui_dependencies)
        self.assertNotIn("codex-realtime-webrtc", locked_packages)
        self.assertNotIn("libwebrtc", locked_packages)
        self.assertNotIn("webrtc-sys", locked_packages)
        self.assertNotIn("webrtc-sys-build", locked_packages)


if __name__ == "__main__":
    unittest.main()

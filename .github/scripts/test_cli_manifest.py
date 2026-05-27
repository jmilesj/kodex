#!/usr/bin/env python3

from __future__ import annotations

import tomllib
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CLI_CARGO_TOML = ROOT / "codex-rs" / "cli" / "Cargo.toml"
CLI_MAIN_RS = ROOT / "codex-rs" / "cli" / "src" / "main.rs"


class CliManifestTest(unittest.TestCase):
    def test_release_cli_does_not_depend_on_app_server_test_client(self) -> None:
        manifest = tomllib.loads(CLI_CARGO_TOML.read_text(encoding="utf-8"))
        dependencies = manifest["dependencies"]
        main_rs = CLI_MAIN_RS.read_text(encoding="utf-8")

        self.assertNotIn("codex-app-server-test-client", dependencies)
        self.assertNotIn("codex_app_server_test_client", main_rs)


if __name__ == "__main__":
    unittest.main()

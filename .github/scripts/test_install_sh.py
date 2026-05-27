#!/usr/bin/env python3

from __future__ import annotations

import os
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
INSTALL_SH = ROOT / "scripts" / "install" / "install.sh"


class InstallShTest(unittest.TestCase):
    def test_linux_targets_use_musl_release_artifacts(self) -> None:
        installer = INSTALL_SH.read_text(encoding="utf-8")

        self.assertIn('vendor_target="aarch64-unknown-linux-musl"', installer)
        self.assertIn('vendor_target="x86_64-unknown-linux-musl"', installer)
        self.assertNotIn('vendor_target="aarch64-unknown-linux-gnu"', installer)
        self.assertNotIn('vendor_target="x86_64-unknown-linux-gnu"', installer)

    def test_up_to_date_local_install_skips_asset_download(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            fake_bin = root / "fake-bin"
            fake_bin.mkdir()
            home = root / "home"
            home.mkdir()
            codex_home = root / "codex-home"
            install_bin = root / "install-bin"
            current = codex_home / "packages" / "standalone" / "current"
            current.mkdir(parents=True)

            curl_log = root / "curl.log"
            write_executable(
                fake_bin / "curl",
                f"""\
                #!/bin/sh
                for arg in "$@"; do
                  case "$arg" in
                    http*) url="$arg" ;;
                  esac
                done
                printf '%s\\n' "$url" >> {curl_log}
                case "$url" in
                  */releases/latest)
                    printf '%s\\n' '{{"tag_name":"kodex-v0.133.0.1779638524"}}'
                    ;;
                  *)
                    echo "unexpected download: $url" >&2
                    exit 42
                    ;;
                esac
                """,
            )
            write_executable(
                current / "kodex",
                """\
                #!/bin/sh
                printf '%s\\n' 'kodex 0.133.0+1779638524'
                """,
            )

            env = {
                **os.environ,
                "CODEX_HOME": str(codex_home),
                "CODEX_INSTALL_DIR": str(install_bin),
                "HOME": str(home),
                "PATH": f"{fake_bin}{os.pathsep}{os.environ['PATH']}",
                "SHELL": "/bin/sh",
            }
            result = subprocess.run(
                ["sh", str(INSTALL_SH)],
                env=env,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                timeout=30,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("already up to date", result.stdout)
            self.assertTrue((install_bin / "kodex").exists())
            self.assertEqual(
                curl_log.read_text(encoding="utf-8").splitlines(),
                ["https://api.github.com/repos/jmilesj/kodex/releases/latest"],
            )


def write_executable(path: Path, content: str) -> None:
    path.write_text(textwrap.dedent(content), encoding="utf-8")
    path.chmod(0o755)


if __name__ == "__main__":
    unittest.main()

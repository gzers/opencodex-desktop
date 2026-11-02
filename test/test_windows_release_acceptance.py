"""Offline guard for the explicit Windows release acceptance version allowlist."""

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[1]


class WindowsReleaseAcceptanceTests(unittest.TestCase):
    def test_current_020_release_is_in_the_installer_acceptance_allowlist(self):
        script = (ROOT / "test/smoke/windows-release-installers.ps1").read_text(encoding="utf-8")
        match = re.search(r"if\(\$Version -notmatch '([^']+)'\)\{throw 'Unsupported release acceptance version'\}", script)
        self.assertIsNotNone(match, "release version must remain explicitly allowlisted")
        allowed = re.compile(match.group(1))
        self.assertRegex("0.2.20", allowed)
        self.assertNotRegex("0.2.21", allowed, "future versions require an explicit acceptance update")

    def test_startup_failure_artifact_locates_sandbox_files_without_contents(self):
        script = (ROOT / "test/smoke/windows-startup.ps1").read_text(encoding="utf-8")
        self.assertIn("sandbox-file-inventory.json", script)
        self.assertIn("relativePath = [IO.Path]::GetRelativePath", script)
        self.assertIn("sha256 = (Get-FileHash", script)
        self.assertNotIn("Get-Content -LiteralPath $file.FullName", script)


if __name__ == "__main__":
    unittest.main()

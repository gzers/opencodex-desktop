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

    def test_each_msi_install_clears_the_product_remembered_install_directory(self):
        script = (ROOT / "test/smoke/windows-installer.ps1").read_text(encoding="utf-8")
        default_case = script.split("# No /D or INSTALLDIR override", 1)[1].split("# MSI reads", 1)[0]
        custom_case = script.split("# MSI reads", 1)[1]
        self.assertIn("Clear-ProductRememberedInstallDirectory", default_case)
        self.assertIn("Clear-ProductRememberedInstallDirectory", custom_case)
        self.assertLess(
            default_case.index("Clear-ProductRememberedInstallDirectory"),
            default_case.index("Invoke-Installer 'msiexec.exe'"),
        )
        self.assertLess(
            custom_case.index("Clear-ProductRememberedInstallDirectory"),
            custom_case.index("Invoke-Installer 'msiexec.exe'"),
        )
        self.assertIn("RegistryView]::Registry32", script)
        self.assertIn("RegistryView]::Registry64", script)
        self.assertIn("DeleteValue('InstallDir',$false)", script)

    def test_installed_packages_start_as_an_ordinary_user_before_admin_uninstall(self):
        script = (ROOT / "test/smoke/windows-installer.ps1").read_text(encoding="utf-8")
        self.assertEqual(script.count("'windows-ordinary-startup.ps1'"), 2)
        self.assertNotIn("'windows-startup.ps1'", script)
        self.assertIn("finally{", script)
        self.assertIn("Invoke-Installer 'msiexec.exe' ('/x", script)


if __name__ == "__main__":
    unittest.main()

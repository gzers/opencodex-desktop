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
        self.assertIn("RegistryHive]::CurrentUser", script)
        self.assertIn("RegistryHive]::LocalMachine", script)
        self.assertIn("RegistryHive]::Users", script)
        self.assertIn("HKEY_USERS\\$subkey", script)
        self.assertIn("Could not clear product InstallDir", script)
        self.assertIn("GetValue('InstallDir',$null)", script)
        self.assertIn("registryCleanup", script)

    def test_all_msi_cells_run_before_default_nsis_can_write_a_remembered_path(self):
        script = (ROOT / "test/smoke/windows-installer.ps1").read_text(encoding="utf-8")
        self.assertIn("$defaultTaskKinds=if($Kind -ceq 'all'){@('msi')}", script)
        default_nsis = script.index("if($Mode -cne 'custom' -and $Kind -ceq 'all')")
        custom_cells = script.index("foreach ($installerKind in $taskKinds)")
        self.assertLess(custom_cells, default_nsis)
        self.assertIn("avoid contaminating MSI AppSearch", script)

    def test_installed_packages_start_as_an_ordinary_user_before_admin_uninstall(self):
        script = (ROOT / "test/smoke/windows-installer.ps1").read_text(encoding="utf-8")
        self.assertEqual(script.count("'windows-ordinary-startup.ps1'"), 2)
        self.assertIn("windows-startup.ps1", script)
        self.assertIn("Default per-user NSIS EXE failed owner-account startup smoke", script)
        self.assertIn("-ExpectedCommit $taskSource -Mode any", script)
        self.assertIn("finally{", script)
        self.assertIn("Invoke-Installer 'msiexec.exe' ('/x", script)

    def test_ci_release_exe_smoke_obeys_the_ordinary_token_boundary(self):
        workflow = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        step = workflow.split("- name: Windows final EXE smoke", 1)[1].split("- name: Record Node24 driver", 1)[0]
        self.assertIn("windows-ordinary-startup.ps1", step)
        self.assertNotIn("windows-startup.ps1", step)
        self.assertIn("if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }", step)

    def test_ordinary_package_smoke_uses_the_package_commit(self):
        installer = (ROOT / "test/smoke/windows-installer.ps1").read_text(encoding="utf-8")
        ordinary = (ROOT / "test/smoke/windows-ordinary-startup.ps1").read_text(encoding="utf-8")
        calls = [line for line in installer.splitlines() if "'windows-ordinary-startup.ps1'" in line]
        self.assertEqual(len(calls), 2)
        for call in calls:
            self.assertIn("-ExpectedCommit $taskSource", call)
        self.assertIn("commit=$ExpectedCommit", ordinary)


if __name__ == "__main__":
    unittest.main()

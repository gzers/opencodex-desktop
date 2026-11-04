"""Offline guards for explicitly admitted Windows release acceptance versions."""

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[1]


class WindowsReleaseAcceptanceTests(unittest.TestCase):
    def test_installer_acceptance_admits_known_releases_through_021_only(self):
        script = (ROOT / "test/smoke/windows-release-installers.ps1").read_text(encoding="utf-8")
        match = re.search(r"if\(\$Version -notmatch '([^']+)'\)\{throw 'Unsupported release acceptance version'\}", script)
        self.assertIsNotNone(match, "release version must remain explicitly allowlisted")
        allowed = re.compile(match.group(1))
        for version in ["0.2.17", "0.2.18", "0.2.19", "0.2.20", "0.2.21"]:
            with self.subTest(version=version):
                self.assertRegex(version, allowed)
        for version in ["0.2.16", "0.2.22", "0.2.210", "0.3.0", "1.0.0",
                        "v0.2.21", "0.2.021", "0.2.21-rc.1", "0.2.21 ", " 0.2.21"]:
            with self.subTest(version=version):
                self.assertNotRegex(version, allowed, "unadmitted versions require an explicit acceptance update")
        self.assertLess(match.end(), script.index("& git fetch"))

    def test_ota_admits_only_explicit_forward_adjacent_releases_through_021(self):
        script = (ROOT / "test/smoke/windows-ota.ps1").read_text(encoding="utf-8")
        match = re.search(r"^if\(!\((.+)\)\)\{throw 'Only the admitted adjacent releases may be tested'\}$", script, re.MULTILINE)
        self.assertIsNotNone(match, "OTA must reject pairs outside its explicit admission guard")
        pair = r"\(\$Version -ceq '(0[.]2[.][0-9]+)' -and \$FromVersion -ceq '(0[.]2[.][0-9]+)'\)"
        self.assertRegex(match.group(1), "^" + pair + "(?: -or " + pair + ")*$")
        admitted = set(re.findall(pair, match.group(1)))
        expected = {"0.2.17": "0.2.16", "0.2.18": "0.2.17", "0.2.19": "0.2.18",
                    "0.2.20": "0.2.19", "0.2.21": "0.2.20"}
        self.assertEqual(admitted, set(expected.items()))
        versions = [f"0.2.{patch}" for patch in range(16, 23)] + ["0.2.210", "0.3.0", "1.0.0",
                    "v0.2.21", "0.2.021", "0.2.21-rc.1", "0.2.21 ", ""]
        for target in versions:
            for baseline in versions:
                with self.subTest(target=target, baseline=baseline):
                    self.assertEqual((target, baseline) in admitted, expected.get(target) == baseline)
        self.assertLess(match.end(), script.index("New-Item"), "reject unadmitted pairs before file or network activity")

    def test_standalone_harness_defaults_target_021_from_020(self):
        installer = (ROOT / "test/smoke/windows-release-installers.ps1").read_text(encoding="utf-8").splitlines()[0]
        ota = (ROOT / "test/smoke/windows-ota.ps1").read_text(encoding="utf-8").splitlines()[0]
        self.assertIn("[string]$Version='0.2.21'", installer)
        self.assertIn("[string]$Version='0.2.21'", ota)
        self.assertIn("[string]$FromVersion='0.2.20'", ota)

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

"""Pure fixtures: no network, installer, product process, or daily configuration."""
import copy
import hashlib
import importlib.util
import io
import json
import re
import xml.etree.ElementTree as ET
from pathlib import Path
import tempfile
import unittest
import zipfile
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("windows023_admit", Path(__file__).with_name("windows023-retention-ci-admit.py"))
admit = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(admit)


class AdmissionTests(unittest.TestCase):
    def setUp(self):
        # Intentionally synthetic candidate pins. They never admit a real run.
        self.pins = {"run": 23, "attempt": 1, "head": "a" * 40, "artifact": 24, "bytes": 123, "sha256": "b" * 64}
        self.run = {"id": 23, "run_attempt": 1, "head_sha": "a" * 40,
                    "repository": {"full_name": admit.REPOSITORY}, "head_repository": {"full_name": admit.REPOSITORY},
                    "path": ".github/workflows/windows-candidate.yml", "event": "workflow_dispatch",
                    "status": "completed", "conclusion": "success"}
        self.artifact = {"id": 24, "expired": False, "name": "opencodex-desktop-0.2.23-windows-x64-candidate",
                         "size_in_bytes": 123, "digest": "sha256:" + "b" * 64,
                         "workflow_run": {"id": 23, "head_sha": "a" * 40}, "created_at": "2026-11-07T00:02:00Z"}
        self.jobs = {"total_count": 1, "jobs": [{"name": "package", "run_id": 23, "run_attempt": 1,
                     "status": "completed", "conclusion": "success", "started_at": "2026-11-07T00:01:00Z",
                     "completed_at": "2026-11-07T00:03:00Z"}]}
        self.ref = {"ref": "refs/tags/v0.2.22", "object": {"type": "tag", "sha": admit.BASELINE_TAG_OBJECT}}
        self.tag = {"tag": "v0.2.22", "sha": admit.BASELINE_TAG_OBJECT,
                    "object": {"type": "commit", "sha": admit.BASELINE_COMMIT}}
        self.release = {"id": admit.BASELINE_RELEASE, "tag_name": "v0.2.22", "draft": False, "prerelease": False,
                        "published_at": "2026-11-06T01:16:24Z", "assets": [self.asset(k) for k in admit.BASELINE]}
        self.env = {"GITHUB_ACTIONS": "true", "RUNNER_OS": "Windows", "RUNNER_ENVIRONMENT": "github-hosted",
                    "GITHUB_REPOSITORY": admit.REPOSITORY, "GITHUB_EVENT_NAME": "workflow_dispatch",
                    "GITHUB_RUN_ATTEMPT": "1", "GITHUB_SHA": "c" * 40}

    def asset(self, kind):
        row = admit.BASELINE[kind]
        return {"id": row["id"], "name": row["name"], "size": row["bytes"], "state": "uploaded",
                "digest": "sha256:" + row["sha256"],
                "browser_download_url": f"https://github.com/{admit.REPOSITORY}/releases/download/v0.2.22/{row['name']}"}

    def test_published_baseline_pins_and_tag(self):
        self.assertEqual(admit.BASELINE_RELEASE, 435948372)
        self.assertEqual(admit.BASELINE["msi"]["id"], 715405068)
        self.assertEqual(admit.BASELINE["nsis"]["id"], 715405138)
        for kind in admit.BASELINE:
            admit.verify_baseline(self.release, self.asset(kind), kind, self.ref, self.tag)

    def test_release_cannot_be_replaced_by_019_draft_or_detached_asset(self):
        for key, value in (("id", 407422226), ("tag_name", "v0.1.9"), ("draft", True),
                           ("prerelease", True), ("published_at", None), ("assets", [])):
            with self.subTest(key=key), self.assertRaises(ValueError):
                admit.verify_baseline({**self.release, key: value}, self.asset("msi"), "msi", self.ref, self.tag)
        release = copy.deepcopy(self.release)
        release["assets"][0]["digest"] = "sha256:" + "0" * 64
        with self.assertRaises(ValueError):
            admit.verify_baseline(release, self.asset("nsis"), "nsis", self.ref, self.tag)

    def test_tag_target_and_asset_identity_are_independent(self):
        for kind in admit.BASELINE:
            for key, value in (("id", 1), ("name", "wrong"), ("size", 0), ("digest", None),
                               ("state", "new"), ("browser_download_url", "https://other.invalid/package")):
                with self.subTest(kind=kind, key=key), self.assertRaises(ValueError):
                    admit.verify_baseline(self.release, {**self.asset(kind), key: value}, kind, self.ref, self.tag)
        for ref, tag in (({**self.ref, "object": {"sha": "0" * 40, "type": "tag"}}, self.tag),
                         (self.ref, {**self.tag, "object": {"sha": "0" * 40, "type": "commit"}})):
            with self.assertRaises(ValueError):
                admit.verify_baseline(self.release, self.asset("msi"), "msi", ref, tag)

    def test_only_exact_successful_package_can_admit(self):
        admit.verify_candidate(self.run, self.artifact, self.jobs, self.pins)
        for key, value in (("id", 1), ("head_sha", "f" * 40), ("run_attempt", 2), ("status", "in_progress"),
                           ("conclusion", "failure"), ("path", ".github/workflows/ci.yml"), ("event", "push"),
                           ("repository", {}), ("head_repository", {})):
            with self.subTest(key=key), self.assertRaises(ValueError):
                admit.verify_candidate({**self.run, key: value}, self.artifact, self.jobs, self.pins)

    def test_artifact_pins_attachment_attempt_time_and_expiry(self):
        for key, value in (("id", 1), ("expired", True), ("expired", None), ("name", "windows-candidate-evidence"),
                           ("size_in_bytes", 0), ("digest", None), ("workflow_run", {}),
                           ("created_at", "2026-11-06T00:00:00Z"), ("created_at", "2026-11-08T00:00:00Z")):
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                admit.verify_candidate(self.run, {**self.artifact, key: value}, self.jobs, self.pins)
        for key, value in (("run_attempt", 2), ("run_id", 1), ("name", "other"), ("conclusion", "skipped")):
            jobs = copy.deepcopy(self.jobs)
            jobs["jobs"][0][key] = value
            with self.subTest(job=key), self.assertRaises(ValueError):
                admit.verify_candidate(self.run, self.artifact, jobs, self.pins)
        with self.assertRaises(ValueError):
            admit.verify_candidate(self.run, self.artifact, {**self.jobs, "total_count": 101}, self.pins)

    def test_local_self_hosted_reruns_and_overrides_stop_before_download(self):
        admit.verify_context(self.env, "nt")
        for key, value in (("RUNNER_ENVIRONMENT", "self-hosted"), ("GITHUB_RUN_ATTEMPT", "2"),
                           ("RUNNER_OS", "macOS"), ("GITHUB_REPOSITORY", "other/repo"),
                           ("GITHUB_EVENT_NAME", "push"), ("GITHUB_SHA", "short"), ("OPENCODEX_SANDBOX", "1")):
            with self.subTest(key=key), self.assertRaises(ValueError):
                admit.verify_context({**self.env, key: value}, "nt")
        with patch.object(admit._http, "download") as download, patch.object(admit.os, "name", "posix"):
            with self.assertRaises(ValueError):
                admit.prepare(None)
            download.assert_not_called()

    def make_archive(self, kind="msi", mutate=None, member=None, extra=None, payload=b"synthetic installer; NEVER executable", product="OpenCodeX Desktop"):
        name = f"{product}_0.2.23_x64{'-setup.exe' if kind == 'nsis' else '_en-US.msi'}"
        row = {"kind": kind, "name": name, "result": "pass", "productVersion": "0.2.23", "firstScreen": "pass",
               "bytes": len(payload), "sha256": hashlib.sha256(payload).hexdigest(),
               "installedExecutableSha256": "d" * 64, "expectedInstalledExecutableSha256": "d" * 64}
        receipt = {"version": "0.2.23", "sourceCommit": self.pins["head"], "result": "pass", "installers": [row]}
        if mutate:
            mutate(receipt)
        stream = io.BytesIO()
        with zipfile.ZipFile(stream, "w") as archive:
            archive.writestr("test/out/windows-candidate/installer-verification.json", json.dumps(receipt))
            archive.writestr(member or "bundle/" + name, payload)
            if extra:
                archive.writestr(*extra)
        stream.seek(0)
        return zipfile.ZipFile(stream)

    def extract(self, archive, kind="msi"):
        with tempfile.TemporaryDirectory(prefix="windows023-admit-fixture-") as temporary:
            root = Path(temporary)
            output, downloads = root / "out", root / "downloads"
            output.mkdir()
            downloads.mkdir()
            with archive:
                target, row = admit.extract_candidate(archive, kind, self.pins["head"], downloads, output)
                self.assertEqual(target.stat().st_size, row["bytes"])
                self.assertTrue((output / "candidate-receipt.original.json").exists())

    def test_extracts_both_pinned_receipted_package_kinds(self):
        for kind in admit.BASELINE:
            self.extract(self.make_archive(kind), kind)

    def test_candidate_uses_raw_packager_name_not_github_asset_rewrite(self):
        # Pinned tauri-bundler 2.9.4 uses settings.product_name() unchanged.
        # Published baseline assets have dots; upload-artifact preserves spaces.
        for kind in admit.BASELINE:
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                self.extract(self.make_archive(kind, product="OpenCodeX.Desktop"), kind)

    def test_receipt_version_source_and_installed_hash_cannot_be_faked(self):
        for key, value in (("version", "0.1.10"), ("sourceCommit", "f" * 40), ("result", "fail"), ("installers", [])):
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.extract(self.make_archive(mutate=lambda receipt: receipt.update({key: value})))
        for key, value in (("sha256", "0" * 64), ("bytes", 0), ("name", "../bad.msi"),
                           ("expectedInstalledExecutableSha256", "0" * 64), ("firstScreen", "fail"),
                           ("productVersion", "0.2.22")):
            with self.subTest(row=key), self.assertRaises(ValueError):
                self.extract(self.make_archive(mutate=lambda receipt: receipt["installers"][0].update({key: value})))

    def test_zip_traversal_windows_aliases_and_duplicate_packages_are_refused(self):
        for member in ("../evil.msi", "/abs.msi", "C:/evil.msi", "a\\evil.msi", "a./evil.msi", "a /evil.msi"):
            with self.subTest(member=member), self.assertRaises(ValueError):
                self.extract(self.make_archive(member=member))
        name = "OpenCodeX Desktop_0.2.23_x64_en-US.msi"
        with self.assertRaises(ValueError):
            self.extract(self.make_archive(extra=("elsewhere/" + name, "duplicate")))
        with self.assertRaises(ValueError):
            self.extract(self.make_archive(extra=("BUNDLE/" + name.upper(), "case alias")))

    def test_archive_symlink_is_rejected(self):
        stream = io.BytesIO()
        with zipfile.ZipFile(stream, "w") as archive:
            info = zipfile.ZipInfo("link")
            info.create_system = 3
            info.external_attr = (0o120777 << 16)
            archive.writestr(info, "target")
        stream.seek(0)
        with zipfile.ZipFile(stream) as archive, self.assertRaises(ValueError):
            admit.checked_zip(archive)


class InstallerTemplateTests(unittest.TestCase):
    root = Path(__file__).resolve().parents[2] / "apps/desktop/tauri"

    def test_nsis_pinned_upstream_and_only_reviewed_delta(self):
        text = (self.root / "windows/nsis023.nsi").read_text()
        self.assertIn("Copyright (c) 2017 - Present Tauri Apps Contributors", text)
        self.assertIn("THE SOFTWARE IS PROVIDED", text)
        upstream = text.split("; OCX023-UPSTREAM-BEGIN\n", 1)[1]
        upstream = upstream.replace('StrCpy $INSTDIR "$LOCALAPPDATA\\Programs\\${PRODUCTNAME}"',
                                    'StrCpy $INSTDIR "$LOCALAPPDATA\\${PRODUCTNAME}"')
        start = upstream.index("Function RestorePreviousInstallLocation\n")
        end = upstream.index("FunctionEnd", start) + len("FunctionEnd")
        upstream = upstream[:start] + '''Function RestorePreviousInstallLocation
  ReadRegStr $4 SHCTX "${MANUPRODUCTKEY}" ""
  StrCmp $4 "" +2 0
    StrCpy $INSTDIR $4
FunctionEnd''' + upstream[end:]
        self.assertEqual(hashlib.sha256(upstream.encode()).hexdigest(),
                         "20f4ecc730defb71f1342eaeaec4021df13be3d843abba0effe88ea5835fa079")
        self.assertIn('${If} $INSTDIR == "${PLACEHOLDER_INSTALL_DIR}"', text)
        self.assertLess(text.index('StrCpy $INSTDIR "$LOCALAPPDATA\\Programs'), text.index("Section Install"))
        self.assertIn('${AndIf} $4 != "$LOCALAPPDATA\\${PRODUCTNAME}"', text)

    def test_wix_pinned_upstream_and_reviewed_delta(self):
        text = (self.root / "windows/wix023.wxs").read_text()
        self.assertIn("Copyright (c) 2017 - Present Tauri Apps Contributors", text)
        self.assertIn("THE SOFTWARE IS PROVIDED", text)
        upstream = text.split("<!-- OCX023-UPSTREAM-BEGIN -->\n", 1)[1]
        upstream = upstream.replace('InstallScope="perUser"\n                 InstallPrivileges="limited"', 'InstallScope="perMachine"')
        start = upstream.index("        <!-- Only the new per-user installation")
        end = upstream.index("        <!-- launch app checkbox -->", start)
        original = r'''        <!-- NOTE: The order of RegistrySearch elements below matters. In WIX, when multiple
             RegistrySearch elements are listed under a single Property, the LAST successful
             match wins. We list the NSIS default-key search first and the MSI InstallDir
             search second so that the MSI-specific path takes priority when both keys exist. -->
        <Property Id="INSTALLDIR">
          <!-- First attempt: Search for the default key value (this is how the nsis installer stores the path) -->
          <RegistrySearch Id="PrevInstallDirNoName" Root="HKCU" Key="Software\\{{manufacturer}}\\{{product_name}}" Type="raw" />

          <!-- Second attempt: Search for "InstallDir" which takes priority if found (this is how the msi installer stores the path) -->
          <RegistrySearch Id="PrevInstallDirWithName" Root="HKCU" Key="Software\\{{manufacturer}}\\{{product_name}}" Name="InstallDir" Type="raw" />
        </Property>

'''
        upstream = upstream[:start] + original + upstream[end:]
        new_directory = '''            <Directory Id="LocalAppDataFolder">
                <Directory Id="OCX023ProgramsFolder" Name="Programs">
                    <Directory Id="INSTALLDIR" Name="{{product_name}}"/>
                </Directory>
            </Directory>'''
        upstream = upstream.replace(new_directory, '''            <Directory Id="$(var.PlatformProgramFilesFolder)" Name="PFiles">
                <Directory Id="INSTALLDIR" Name="{{product_name}}"/>
            </Directory>''')
        upstream = upstream.replace('''<Component Id="RegistryEntries" Guid="*">
                <RegistryKey Root="HKCU" Key="Software\\\\{{manufacturer}}\\\\{{product_name}}\\023">''',
                                    '''<Component Id="RegistryEntries" Guid="*">
                <RegistryKey Root="HKCU" Key="Software\\\\{{manufacturer}}\\\\{{product_name}}">''')
        upstream = upstream.replace('<!-- Per-user protocol registration uses the installing user. -->', '<!-- Change the Root to HKCU for perUser installations -->')
        upstream = upstream.replace('<RegistryKey Root="HKCU" Key="Software\\Classes\\\\{{protocol}}">', '<RegistryKey Root="HKLM" Key="Software\\Classes\\\\{{protocol}}">')
        upstream = re.sub(r'(<File Id="Path" Source="\{\{main_binary_path\}\}") Checksum="yes"/>\n\s*<RegistryValue[^\n]+Name="Desktop"[^\n]+/>',
                          r'\1 KeyPath="yes" Checksum="yes"/>', upstream)
        upstream = re.sub(r'(<File Id="Bin_\{\{ bin.id \}\}" Source="\{\{bin.path\}\}")/>\n\s*<RegistryValue[^\n]+Name="\{\{bin.id\}\}"[^\n]+/>',
                          r'\1 KeyPath="yes"/>', upstream)
        upstream = re.sub(r'            <!-- OCX023-PAYLOAD-BEGIN.*?<!-- OCX023-PAYLOAD-END -->', '            {{resources}}', upstream, flags=re.S)
        upstream = re.sub(r'            <!-- OCX023-RESOURCE-REFS-BEGIN -->.*?<!-- OCX023-RESOURCE-REFS-END -->',
                          '''            {{#each resource_file_ids as |resource_file_id| ~}}
                <ComponentRef Id="{{ resource_file_id }}"/>
            {{/each~}}''', upstream, flags=re.S)
        upstream = upstream.replace('\n                <RemoveFolder Id="OCX023RemovePrograms" Directory="OCX023ProgramsFolder" On="uninstall" />', '')
        self.assertEqual(hashlib.sha256(upstream.encode()).hexdigest(),
                         "e371a01628a06730828f9bd24111feacb8bec53c250ccec4b46df756fe0a0198")

    def test_same_ui_and_actual_user_programs_directory_and_custom_choice(self):
        config = json.loads((self.root / "tauri.windows.conf.json").read_bytes())["bundle"]["windows"]
        self.assertEqual(config["nsis"]["installMode"], "currentUser")
        self.assertEqual(config["nsis"]["template"], "windows/nsis023.nsi")
        self.assertEqual(config["wix"]["template"], "windows/wix023.wxs")
        text = (self.root / config["wix"]["template"]).read_text()
        fragment = re.search(r'<Directory Id="LocalAppDataFolder">.*?</Directory>\s*</Directory>', text, re.S).group()
        directory = ET.fromstring(fragment)
        self.assertEqual(directory.find("Directory").attrib["Name"], "Programs")
        self.assertEqual(directory.find("Directory/Directory").attrib["Id"], "INSTALLDIR")
        self.assertIn('<Property Id="WIXUI_INSTALLDIR" Value="INSTALLDIR"', text)
        self.assertIn('NOT INSTALLDIR AND OCX023_REMEMBERED_I', text)
        self.assertIn('Sequence="both"', text)
        self.assertIn('InstallScope="perUser"', text)
        self.assertIn('InstallPrivileges="limited"', text)
        self.assertIn('先卸载旧管理器（保留配置），再安装新版', text)
        self.assertIn('NOT ALLUSERS', text)
        self.assertNotIn('PrevInstallDirNoName', text)
        self.assertEqual(text.count('{C5476FDD-CB86-46AF-A97F-96DD56A32CA0}'), 2)
        self.assertIn('Win64="yes"', text)
        self.assertIn('Win64="no"', text)

    def test_complete_fixed_payload_uses_registry_keypaths_without_ice_suppression(self):
        text = (self.root / "windows/wix023.wxs").read_text()
        fragment = re.search(r'<Directory Id="OCX023RuntimeBundle".*?<!-- OCX023-PAYLOAD-END -->', text, re.S).group().split('<!-- OCX023-PAYLOAD-END -->')[0].strip()
        directory = ET.fromstring(fragment)
        component = directory.find("Component")
        files = {row.attrib["Source"].rsplit('\\', 1)[-1] for row in component.findall("File")}
        self.assertEqual(files, {"runtime-bundle-0.2.23-win32-x64.tar.gz", "maintained-runtime.tgz",
                                 "runtime-bundle.manifest.json", "runtime-bundle.manifest.sha256"})
        self.assertEqual(component.find("RegistryValue").attrib["Root"], "HKCU")
        self.assertEqual(component.find("RegistryValue").attrib["KeyPath"], "yes")
        self.assertTrue(component.find("RemoveFolder") is not None)
        self.assertNotIn('{{resources}}', text)
        self.assertNotIn('SuppressIces', text)
        self.assertNotIn('RemoveFile', text)
        self.assertNotIn('RemoveFolderEx', text)

    def test_multiple_file_registry_keypath_components_have_fixed_distinct_guids(self):
        text = (self.root / "windows/wix023.wxs").read_text()
        payload = text.split('<!-- OCX023-PAYLOAD-BEGIN', 1)[1].split('<!-- OCX023-PAYLOAD-END -->', 1)[0]
        expected = {
            "OCX023Documents": "9501D6A2-BA37-53EA-B53A-10CB8E34412E",
            "OCX023RuntimeBundle": "7F3ED10E-ADBA-56E4-B6A1-EFC692135F8C",
        }
        for name, guid in expected.items():
            match = re.search(r'<Component Id="' + name + r'".*?</Component>', payload, re.S)
            component = ET.fromstring(match.group())
            self.assertEqual(component.attrib["Guid"], guid)
            self.assertGreater(len(component.findall("File")), 1)
            self.assertEqual(component.find("RegistryValue").attrib["KeyPath"], "yes")
            self.assertEqual(component.find("RegistryValue").attrib["Root"], "HKCU")
        self.assertEqual(len(set(expected.values())), len(expected))

    def test_nsis_replaces_only_registered_022_program_and_never_purges_data(self):
        hooks = (self.root / "windows/installer-hooks.nsh").read_text()
        pre = hooks.split('!macro NSIS_HOOK_PREINSTALL\n')[1].split('!macroend')[0]
        self.assertIn('ReadRegStr $4 HKCU "${UNINSTKEY}" "DisplayVersion"', pre)
        self.assertIn('${If} $4 == "0.2.22"', pre)
        self.assertIn('ReadRegStr $6 HKCU "${UNINSTKEY}" "UninstallString"', pre)
        self.assertIn('/S /UPDATE _?=$LOCALAPPDATA', pre)
        self.assertNotIn('RMDir', pre)
        self.assertNotIn('CopyFiles', pre)
        self.assertNotIn('EnumRegKey', pre)
        self.assertNotIn('.codex', pre)
        self.assertNotIn('/D=', pre)
        self.assertIn('StrCpy $DeleteAppDataCheckboxState 0', hooks)


if __name__ == "__main__":
    unittest.main(verbosity=2)

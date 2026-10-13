import importlib.util
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location('caption_probe', Path(__file__).parent / 'smoke/windows_caption_probe.py')
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


class FakeWindows:
    def __init__(self):
        self.identities = [{'sha256': 'a' * 64, 'ownerPid': 10904, 'creationFileTime': 123}] * 2
        self.queries = []
        self.caption_ok = True
        self.hit_ok = True

    def identity(self, pid, hwnd):
        return self.identities.pop(0)

    def window_state(self, hwnd):
        self.queries.append('window')
        return {'windowRect': [-7, -7, 3206, 1150], 'maximized': True}

    def caption(self, hwnd):
        self.queries.append('caption')
        slots = [{'rect': [0, 0, 0, 0], 'stateFlags': 0} for _ in range(6)]
        for name, slot in probe.BUTTON_SLOTS.items():
            slots[slot] = {'rect': [2920 + slot * 30, 0, 2940 + slot * 30, 31], 'stateFlags': 0}
        return {'ok': self.caption_ok, 'lastError': 0 if self.caption_ok else 5, 'slots': slots if self.caption_ok else []}

    def hit_test(self, hwnd, packed):
        self.queries.append(packed)
        return {'ok': self.hit_ok, 'value': 9 if self.hit_ok else None, 'lastError': 0 if self.hit_ok else 5}

    def point_target(self, pid, hwnd, x, y):
        self.queries.append(('point', x, y))
        return {'windowFound': True, 'rootMatchesTarget': True, 'belongsToTargetPid': True}


class CaptionObservationTests(unittest.TestCase):
    def test_signed_screen_coordinates_round_trip(self):
        for x, y in [(-32768, -32768), (32767, 32767), (-7, -7), (-1453, 211), (3199, 15)]:
            packed = probe.pack_screen_point(x, y)
            self.assertEqual(probe.ctypes.c_int16(packed & 0xffff).value, x)
            self.assertEqual(probe.ctypes.c_int16(packed >> 16).value, y)

    def test_overflow_and_noninteger_are_rejected_instead_of_wrapping_to_another_button(self):
        for xy in [(32768, 0), (0, -32769), (1.5, 1), (True, 1)]:
            with self.assertRaises(ValueError):
                probe.pack_screen_point(*xy)

    def test_points_stay_inside_caption_on_negative_origin_monitor(self):
        points = probe.caption_points([-95, -7, -49, 24], [-1000, -7, 0, 760])
        self.assertEqual(len(points), 3)
        self.assertTrue(all(-95 <= x < -49 and -7 <= y < 24 for x, y in points))

    def test_invalid_or_off_window_button_bounds_never_issue_hit_test(self):
        for rect in [[1, 1, 1, 2], [3, 4, 1, 2], [-8, 0, 10, 20], [0, 0, 3207, 20]]:
            self.assertEqual(probe.caption_points(rect, [-7, -7, 3206, 1150]), [])

    def test_single_pixel_rect_has_one_unique_sample(self):
        self.assertEqual(probe.caption_points([1, 2, 2, 3], [0, 0, 10, 10]), [(1, 2)])

    def test_sha_mismatch_blocks_all_window_queries(self):
        native = FakeWindows()
        with self.assertRaisesRegex(ValueError, 'SHA256 mismatch'):
            probe.collect_snapshot(native, 10904, 6685560, 'b' * 64)
        self.assertEqual(native.queries, [])

    def test_reused_process_identity_is_rejected_before_queries(self):
        native = FakeWindows()
        with self.assertRaisesRegex(ValueError, 'creation time mismatch'):
            probe.collect_snapshot(native, 10904, 6685560, 'a' * 64, 456)
        self.assertEqual(native.queries, [])

    def test_identity_changed_during_queries_cannot_be_accepted(self):
        native = FakeWindows()
        native.identities[1] = {**native.identities[1], 'creationFileTime': 456}
        with self.assertRaisesRegex(ValueError, 'identity changed'):
            probe.collect_snapshot(native, 10904, 6685560, 'a' * 64)

    def test_unavailable_caption_query_produces_no_invented_button_geometry(self):
        native = FakeWindows()
        native.caption_ok = False
        result = probe.collect_snapshot(native, 10904, 6685560, 'a' * 64)
        self.assertEqual(native.queries, ['window', 'caption'])
        self.assertFalse(result['captionQuery']['ok'])
        self.assertTrue(all(item['titleBarInfoEx'] is None and not item['probes'] for item in result['buttons'].values()))

    def test_query_failure_is_not_a_successful_client_or_caption_hit(self):
        native = FakeWindows()
        native.hit_ok = False
        result = probe.collect_snapshot(native, 10904, 6685560, 'a' * 64)
        for item in result['buttons'].values():
            for hit in item['probes']:
                self.assertFalse(hit['ok'])
                self.assertIsNone(hit['value'])
                self.assertEqual(hit['lastError'], 5)

    def test_observations_are_not_inputs_or_acceptance_claims(self):
        native = FakeWindows()
        result = probe.collect_snapshot(native, 10904, 6685560, 'A' * 64, 123)
        self.assertEqual(len(native.queries), 20)
        self.assertTrue(all(hit['windowUnderPoint']['rootMatchesTarget'] for item in result['buttons'].values() for hit in item['probes']))
        self.assertFalse(result['inputIssued'])
        self.assertFalse(result['applicationLifecycleChanged'])
        self.assertFalse(result['acceptancePassed'])
        self.assertTrue(result['window']['maximized'])

    def test_win32_struct_layouts_are_fixed_width_on_non_windows_test_hosts(self):
        self.assertEqual(probe.ctypes.sizeof(probe.RECT), 16)
        self.assertEqual(probe.ctypes.sizeof(probe.TITLEBARINFOEX), 140)
        self.assertEqual(probe.ctypes.sizeof(probe.WINDOWPLACEMENT), 44)

    def test_correct_parent_hit_cannot_disguise_another_window_at_the_point(self):
        native = FakeWindows()
        native.point_target = lambda *args: {'windowFound': True, 'rootMatchesTarget': False, 'belongsToTargetPid': False}
        result = probe.collect_snapshot(native, 10904, 6685560, 'a' * 64)
        for item in result['buttons'].values():
            for hit in item['probes']:
                self.assertEqual(hit['value'], 9)
                self.assertFalse(hit['windowUnderPoint']['rootMatchesTarget'])
        self.assertFalse(result['acceptancePassed'])

    def point_backend(self, actual, root, owner):
        class User:
            class_reads = 0

            def WindowFromPoint(self, point):
                return actual

            def GetAncestor(self, window, flag):
                self.last_ancestor_flag = flag
                return root

            def GetWindowThreadProcessId(self, window, destination):
                destination._obj.value = owner
                return 99

            def GetClassNameW(self, window, destination, size):
                self.class_reads += 1
                destination.value = 'TestChild'
                return len(destination.value)
        native = probe.NativeWindows.__new__(probe.NativeWindows)
        native.user = User()
        return native

    def test_child_of_target_root_is_recorded_without_claiming_input(self):
        native = self.point_backend(22, 6685560, 10904)
        result = native.point_target(10904, 6685560, -7, 15)
        self.assertTrue(result['rootMatchesTarget'])
        self.assertEqual(result['targetFamilyHwnd'], 22)
        self.assertEqual(result['targetFamilyClass'], 'TestChild')
        self.assertEqual(native.user.last_ancestor_flag, 2)

    def test_unrelated_window_does_not_disclose_class_or_identity(self):
        native = self.point_backend(22, 33, 456)
        result = native.point_target(10904, 6685560, 3190, 15)
        self.assertFalse(result['rootMatchesTarget'])
        self.assertFalse(result['belongsToTargetPid'])
        self.assertIsNone(result['targetFamilyHwnd'])
        self.assertIsNone(result['targetFamilyClass'])
        self.assertEqual(native.user.class_reads, 0)

    def test_missing_window_under_point_is_explicit(self):
        native = self.point_backend(None, None, 0)
        result = native.point_target(10904, 6685560, 3190, 15)
        self.assertFalse(result['windowFound'])
        self.assertFalse(result['rootMatchesTarget'])
        self.assertIsNone(result['targetFamilyHwnd'])
        self.assertEqual(native.user.class_reads, 0)


if __name__ == '__main__':
    unittest.main()

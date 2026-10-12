"""Synthetic guard checks only; does not inspect or operate any app UI."""
import datetime
import json
from pathlib import Path
import subprocess
import sys
import time
from run_guard import classify, validate_finished, stop_owned, stream_timeout, read_available

def sample(elapsed=.2, front=True, visible=True):
    return dict(kind='sample', elapsed_s=elapsed, target_frontmost=front, target_window_on_screen=visible)

cases = []
def check(name, actual, expected):
    assert actual == expected, (name, actual, expected)
    cases.append(dict(name=name, passed=True, result=actual))

check('valid_sample', classify(sample(), .1), None)
check('focus_loss', classify(sample(front=False), .1), 'foreground_or_visible_window_lost')
check('hidden_window', classify(sample(visible=False), .1), 'foreground_or_visible_window_lost')
check('sampling_gap', classify(sample(1.2), .1), 'sampling_clock_or_gap_invalid')
check('clock_backwards', classify(sample(.05), .1), 'sampling_clock_or_gap_invalid')
check('clock_repeated', classify(sample(.1), .1), 'sampling_clock_or_gap_invalid')
check('nonfinite_clock', classify(sample(float('nan')), .1), 'invalid_sample_clock')
check('readiness_timeout', classify(dict(kind='invalid',reason='not_ready_within_ten_seconds')), 'not_ready_within_ten_seconds')
valid = [dict(kind='ready'), sample(.1), sample(.2), dict(kind='stopped',armed=True)]
stable = dict(process_identity_stable=True)
check('valid_completed_stream', validate_finished(valid, stable), None)
check('missing_resource_summary', validate_finished(valid, None), 'resource_summary_missing')
check('helper_identity_change', validate_finished(valid, dict(process_identity_stable=False)), 'resource_process_identity_changed')
check('missing_stop_boundary', validate_finished(valid[:-1], stable), 'missing_or_repeated_monitor_boundaries')
check('missing_readiness', validate_finished(valid[1:], stable), 'missing_or_repeated_monitor_boundaries')
check('repeated_readiness', validate_finished(valid[:-1]+[dict(kind='ready'), valid[-1]], stable), 'missing_or_repeated_monitor_boundaries')
check('unarmed_stop', validate_finished(valid[:-1]+[dict(kind='stopped',armed=False)], stable), 'monitor_stopped_without_readiness')
check('loss_in_buffered_tail', validate_finished(valid+[sample(.3,front=False)], stable), 'foreground_or_visible_window_lost')
child = subprocess.Popen([sys.executable,'-c','import time;time.sleep(30)'])
started = time.monotonic()
try:
    stop_owned(child)
    check('stop_only_owned_collector', child.poll() is not None, True)
    check('owned_child_stop_bounded', time.monotonic()-started < 2.5, True)
finally:
    stop_owned(child)
check('negative_clock', classify(sample(-1)), 'invalid_sample_clock')
check('boolean_clock', classify(sample(True)), 'invalid_sample_clock')
check('event_after_stop', validate_finished(valid+[sample(.3)], stable), 'event_after_stop_boundary')
check('watchdog_healthy', stream_timeout(True, 9.5, 0, 10), None)
check('watchdog_stall', stream_timeout(True, 8.9, 0, 10), 'monitor_stream_stalled_over_one_second')
check('watchdog_not_ready', stream_timeout(False, 0, 0, 12.1), 'monitor_readiness_watchdog_timeout')
check('readiness_window', stream_timeout(False, 0, 0, 11.9), None)
report = dict(captured_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),
              scope='synthetic harness checks; no application, foreground or performance acceptance',
              count=len(cases), failed=0, cases=cases)
target=Path(sys.argv[1]) if len(sys.argv)>1 else Path(__file__).with_name('selfcheck-v2.json')
with target.open('x') as stream:
    json.dump(report,stream,indent=2);stream.write('\n')
print(json.dumps(dict(count=len(cases),failed=0,report=str(target))))

"""External diagnostic harness; never launches or activates an application."""
import argparse
import datetime
import hashlib
import json
import math
import os
from pathlib import Path
import selectors
import subprocess
import sys
import time

BASE = Path('/private/tmp/ocx-0110-performance-20261012')
HERE = Path(__file__).resolve().parent

def write_json(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, indent=2, ensure_ascii=False)
        stream.write('\n')

def settings(config):
    raw = (Path(config['root']) / 'manager-state/preferences.json').read_bytes()
    value = json.loads(raw)
    return dict(appearance=value['appearance'], updates=value['updates'],
                preferences_sha256=hashlib.sha256(raw).hexdigest(),
                captured_at=datetime.datetime.now(datetime.timezone.utc).isoformat())

def classify(event, previous=None):
    if event.get('kind') == 'invalid':
        return event['reason']
    if event.get('kind') != 'sample':
        return None
    elapsed = event.get('elapsed_s')
    if isinstance(elapsed, bool) or not isinstance(elapsed, (int, float)) or not math.isfinite(elapsed) or elapsed < 0:
        return 'invalid_sample_clock'
    if event.get('target_frontmost') is not True or event.get('target_window_on_screen') is not True:
        return 'foreground_or_visible_window_lost'
    if previous is not None and (elapsed <= previous or elapsed-previous > 1):
        return 'sampling_clock_or_gap_invalid'
    return None

def stop_owned(child):
    if child is not None and child.poll() is None:
        child.terminate()
        try:
            child.wait(timeout=2)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait(timeout=2)

def validate_finished(events, summary):
    armed = False
    previous = None
    ready_count = 0
    stopped_count = 0
    for event in events:
        if event.get('kind') == 'invalid':
            return event['reason']
        if event.get('kind') == 'ready':
            armed = True
            ready_count += 1
        if armed:
            reason = classify(event, previous)
            if reason: return reason
            if stopped_count:
                return 'event_after_stop_boundary'
            if event.get('kind') == 'sample': previous = event['elapsed_s']
        if event.get('kind') == 'stopped':
            stopped_count += 1
            if event.get('armed') is not True: return 'monitor_stopped_without_readiness'
    if ready_count != 1 or stopped_count != 1 or previous is None:
        return 'missing_or_repeated_monitor_boundaries'
    if summary is None: return 'resource_summary_missing'
    if summary.get('process_identity_stable') is not True: return 'resource_process_identity_changed'
    return None

def read_available(pipe, pending):
    """Use raw reads so TextIO buffering cannot hide lines from the selector."""
    block = os.read(pipe.fileno(), 65536)
    if not block:
        if pending: raise ValueError('truncated_monitor_line')
        return [], pending, True
    pending += block
    lines = pending.split(bytes([10]))
    pending = lines.pop()
    return [json.loads(line) for line in lines], pending, False

def stream_timeout(ready, last_received, started, now):
    if ready and now-last_received > 1:
        return 'monitor_stream_stalled_over_one_second'
    if not ready and now-started > 12:
        return 'monitor_readiness_watchdog_timeout'
    return None

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('phase')
    parser.add_argument('--pid', required=True, type=int)
    parser.add_argument('--quality', required=True, choices=['high','mid','low'])
    parser.add_argument('--renderer', required=True, choices=['mesh','css'])
    parser.add_argument('--duration', type=int, default=180)
    args = parser.parse_args()
    if not args.phase.replace('-', '').replace('_', '').isalnum():
        parser.error('phase must be a fresh simple directory name')
    if args.duration < 10 or args.duration > 600 or args.duration % 10:
        parser.error('duration must be a multiple of 10 within 10..600')
    sys.path.insert(0, str(BASE))
    import measure
    config = measure.CONFIG['candidate0110']
    expected = dict(theme='light', visual_effects=args.quality, glow_render=args.renderer, interface_scale=100)
    actual = settings(config)
    assert actual['appearance'] == expected, ('actual settings mismatch', actual['appearance'], expected)
    assert hashlib.sha256(Path(config['exe']).read_bytes()).hexdigest() == config['test_exe_sha256'], 'test executable drift'
    apps = measure.all_apps()
    tests = [item for item in apps if item['exe'] != measure.DAILY]
    assert len(tests) == 1 and tests[0]['pid'] == args.pid and tests[0]['exe'] == config['exe'], 'single existing test instance required'
    out = BASE / 'candidate0110' / args.phase
    out.mkdir()  # Refuse overwriting any prior run.
    write_json(out/'settings-before.json', actual)
    write_json(out/'guard-context.json', dict(
        source=config['source_sha'], exe_sha256=config['test_exe_sha256'],
        process=tests[0], expected=expected, original_manifest_appearance=config['appearance'],
        validation_role='attribution diagnostic only; full start/end UI and human review remain required',
        monitoring='0.1s NSWorkspace/CGWindow snapshots; gaps or stream stalls >1s invalidate; no app UI actions',
        warmup_seconds=90, requested_measurement_seconds=args.duration))
    stop = out/'foreground.stop'
    collector = None
    ready_at = None
    previous = None
    reason = None
    completed = False
    with (out/'foreground.jsonl').open('x') as events, (out/'monitor.stderr.log').open('x') as errors, (out/'collector.stdout.log').open('x') as output, (out/'collector.stderr.log').open('x') as collector_errors:
        monitor = subprocess.Popen([str(HERE/'foreground-stream'), str(args.pid), str(stop), str(args.duration+120)], stdout=subprocess.PIPE, stderr=errors, bufsize=0)
        os.set_blocking(monitor.stdout.fileno(), False)
        pending = b''
        monitor_started = last_received = time.monotonic()
        selector = selectors.DefaultSelector()
        selector.register(monitor.stdout, selectors.EVENT_READ)
        try:
            while True:
                for key, _ in selector.select(timeout=.1):
                    batch, pending, eof = read_available(key.fileobj, pending)
                    if eof:
                        reason = 'monitor_stream_ended'
                        break
                    if batch: last_received = time.monotonic()
                    for event in batch:
                        events.write(json.dumps(event, sort_keys=True)+'\n'); events.flush()
                        if event['kind'] == 'ready':
                            if ready_at is not None:
                                reason = 'repeated_readiness_boundary'
                                break
                            ready_at = time.monotonic()
                            write_json(out/'launch.json', dict(pid=args.pid, monotonic_start=ready_at, scene_preparation_at=datetime.datetime.now(datetime.timezone.utc).isoformat(), note='existing process; scene warmup only, not app launch'))
                            print('foreground ready; 90s scene warmup', flush=True)
                        if event['kind'] == 'sample' and event.get('target_pid') != args.pid:
                            reason = 'monitor_target_pid_changed'
                        elif ready_at is not None:
                            reason = classify(event, previous)
                            if event['kind'] == 'sample': previous = event['elapsed_s']
                        elif event['kind'] == 'invalid':
                            reason = event['reason']
                        if reason: break
                    if reason: break
                reason = reason or stream_timeout(ready_at is not None, last_received, monitor_started, time.monotonic())
                if reason: break
                if monitor.poll() is not None:
                    reason = 'monitor_exited_before_collection_completed'
                    break
                if ready_at is not None and collector is None and time.monotonic()-ready_at >= 90:
                    assert tests[0] in measure.all_apps(), 'test process identity changed during warmup'
                    # Pin the collector to real persisted settings, retaining the stale original manifest above.
                    script = 'import json,sys;sys.path.insert(0,sys.argv[1]);import measure;c=measure.CONFIG["candidate0110"];s=json.load(open(sys.argv[2]));c["appearance"]=s["appearance"];c["updates"]=s["updates"];measure.observe("candidate0110",sys.argv[3],int(sys.argv[4]))'
                    collector = subprocess.Popen([sys.executable, '-c', script, str(BASE), str(out/'settings-before.json'), args.phase, str(args.duration)], stdout=output, stderr=collector_errors)
                    print('resource measurement started', flush=True)
                if collector is not None and collector.poll() is not None:
                    completed = collector.returncode == 0
                    if not completed: reason = 'resource_collector_failed'
                    break
        except Exception as error:
            reason = 'harness_error:' + type(error).__name__
            print(reason, file=sys.stderr, flush=True)
        finally:
            stop_owned(collector)
            stop.touch(exist_ok=False)
            try: monitor.wait(timeout=2)
            except subprocess.TimeoutExpired: stop_owned(monitor)
            os.set_blocking(monitor.stdout.fileno(), True)
            tail = pending + monitor.stdout.read()
            events.write(tail.decode('utf-8')); events.flush()
            selector.close()
    if tests[0] not in measure.all_apps(): reason = reason or 'test_process_identity_changed'
    after = settings(config)
    write_json(out/'settings-after.json', after)
    all_events = [json.loads(line) for line in (out/'foreground.jsonl').read_text().splitlines()]
    summary_exists = (out/'summary.json').is_file()
    resource_summary = json.loads((out/'summary.json').read_bytes()) if summary_exists else None
    reason = reason or validate_finished(all_events, resource_summary)
    if actual['preferences_sha256'] != after['preferences_sha256']: reason = reason or 'preferences_changed'
    verdict = dict(collection_completed=completed, foreground_monitor_exit=monitor.returncode,
                   resource_collector_exit=collector.returncode if collector else None,
                   invalid_reason=reason, diagnostic_collection_valid=completed and monitor.returncode == 0 and not reason and summary_exists,
                   application_acceptance='not_evaluated', full_start_end_ui_review='required',
                   partial_samples='forensic only when invalid; never a successful performance run')
    write_json(out/'guard-result.json', verdict)
    print(json.dumps(verdict), flush=True)
    return 0 if verdict['diagnostic_collection_valid'] else 2

if __name__ == '__main__':
    raise SystemExit(main())

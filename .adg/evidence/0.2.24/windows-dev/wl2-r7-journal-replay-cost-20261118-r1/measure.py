from pathlib import Path
from statistics import median
import json

root = Path(__file__).resolve().parent
case = root / 'direct-wl2_r7_journal_replay_cost_tracks_native_prefix_and_pure_data_separately'
stdout = (case / 'stdout.log').read_text(encoding='utf-8').splitlines()


def event(prefix):
    values = [json.loads(line.split(prefix, 1)[1]) for line in stdout if prefix in line]
    assert len(values) == 1, prefix
    return values[0]


rows = event('r7-recovery-timing=')
pure = event('r7-journal-pure-data=')
replays = [row for row in rows if row['operation'] == 'journal-append-replay']
reads = [row for row in rows if row['operation'] == 'journal-records']
assert len(replays) >= 8 and len(reads) == 2 * len(replays) + 1
assert [r['ordinal'] for r in rows] == list(range(1, len(rows) + 1))
assert all(r['completed'] and r['remainingUs'] > 0 and r['rawOsError'] is None for r in rows)
for index, replay in enumerate(replays):
    before, _, after = rows[index * 3:index * 3 + 3]
    assert before['operation'] == after['operation'] == 'journal-records'
    assert before['expectedObjects'] == index + 2
    assert replay['expectedObjects'] == index + 1
    assert after['expectedObjects'] == index + 3
    assert before['startedUs'] + before['elapsedUs'] <= replay['startedUs']
    assert replay['startedUs'] + replay['elapsedUs'] <= after['startedUs']
assert rows[-1]['operation'] == 'journal-records'


def stats(items):
    times = [item['elapsedUs'] for item in items]
    return {'count': len(times), 'sumUs': sum(times), 'medianUs': median(times),
            'minUs': min(times), 'maxUs': max(times)}


replay = stats(replays)
native = stats(reads[:-1])
iterations = pure['iterations']
policy = pure['policyDigestUs'] / iterations
work = sum(row['expectedObjects'] for row in replays)
result = {'scope': 'One debug MSVC native Journal with synthetic node-event transitions. The independent final check is excluded from the append-only comparison.',
          'appends': len(replays), 'originalDeadlineSeconds': 10,
          'lastCheckRemainingUs': rows[-1]['remainingUs'], 'pureCalibration': pure,
          'independentReplay': replay, 'nativeJournalReadback': native,
          'finalIndependentCheck': reads[-1], 'replayedEntriesIncludingProposed': work,
          'replayToNativeReadbackPercent': 100 * replay['sumUs'] / native['sumUs'],
          'estimatedPolicyDigestUsPerCall': policy,
          'estimatedEventEncodeAndDigestUsPerCall': pure['eventEncodeAndDigestUs'] / iterations,
          'estimatedPolicyShareOfReplayPercent': 100 * policy * work / replay['sumUs'],
          'scopeLimits': ['Policy share is an estimate from a separate tight loop, not an in-path timer.',
                          'Native journal readback includes byte validation and typed Current checks.',
                          'Append preparation, writing, immediate readback, and CAS are outside these two timers.',
                          'The sample neither qualifies production nor projects a complete recovery budget.']}
with (root / 'measurement.json').open('x', encoding='utf-8', newline='\n') as handle:
    json.dump(result, handle, ensure_ascii=False, indent=2)
    handle.write('\n')
print(json.dumps(result))

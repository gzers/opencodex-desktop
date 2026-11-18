from collections import defaultdict
import json
from pathlib import Path
from statistics import median

root = Path(__file__).resolve().parent
case = root / 'direct-wl2_r7_journal_replay_cost_tracks_native_prefix_and_pure_data_separately'
lines = (case / 'stdout.log').read_text(encoding='utf-8').splitlines()
prefix = 'r7-recovery-timing='
traces = [json.loads(line.split(prefix, 1)[1]) for line in lines if prefix in line]
assert len(traces) == 1
rows = traces[0]
assert [row['ordinal'] for row in rows] == list(range(1, len(rows) + 1))
assert all(row['completed'] and row['remainingUs'] > 0 and row['rawOsError'] is None for row in rows)

head = ['journal-context-native']
pair = ['journal-record-fresh', 'journal-record-retained-read']
tail = ['journal-current-native', 'journal-current-bytes', 'journal-current-typed', 'journal-membership']
breakdown = defaultdict(list)
parents, sequence, pending = [], [], []
for row in rows:
    kind = row['operation']
    if kind == 'journal-append-replay':
        assert not pending
        sequence.append(row)
        continue
    if kind != 'journal-records':
        pending.append(row)
        continue
    records = row['expectedObjects'] - 1
    assert [element['operation'] for element in pending] == head + pair * records + tail
    assert sum(p['expectedBytes'] for p in pending if p['operation'] in (pair[1], tail[1])) == row['expectedBytes']
    assert pending[-1]['expectedObjects'] == row['expectedObjects']
    limit = row['startedUs'] + row['elapsedUs']
    assert all(row['startedUs'] <= p['startedUs'] <= p['startedUs'] + p['elapsedUs'] <= limit for p in pending)
    assert all(a['startedUs'] + a['elapsedUs'] <= b['startedUs'] for a, b in zip(pending, pending[1:]))
    for element in pending:
        breakdown[element['operation']].append(element['elapsedUs'])
    parents.append(row)
    sequence.append(row)
    pending.clear()
assert not pending
appends = (len(parents) - 1) // 2
assert appends >= 8 and len(parents) == 2 * appends + 1 and len(sequence) == 3 * appends + 1
for index in range(appends):
    before, replay, after = sequence[3 * index:3 * index + 3]
    assert [r['operation'] for r in (before, replay, after)] == ['journal-records', 'journal-append-replay', 'journal-records']
    assert [r['expectedObjects'] for r in (before, replay, after)] == [index + 2, index + 1, index + 3]
assert sequence[-1]['expectedObjects'] == appends + 2

def summarize(values):
    return {'count': len(values), 'sumUs': sum(values), 'medianUs': median(values)}

stages = {label: summarize(breakdown[label]) for label in head + pair + tail}
parent_total = sum(row['elapsedUs'] for row in parents)
records_total = sum(stages[label]['sumUs'] for label in pair)
current_bytes_and_typed = sum(stages[label]['sumUs'] for label in tail[1:3])
result = {'appends': appends, 'wholeReadbacks': len(parents), 'originalDeadlineSeconds': 10,
          'lastCheckRemainingUs': parents[-1]['remainingUs'], 'wholeReadbackUs': parent_total,
          'aggregateExpectedBytes': sum(row['expectedBytes'] for row in parents), 'stages': stages,
          'residualUs': parent_total - sum(stage['sumUs'] for stage in stages.values()),
          'perRecordPercent': 100 * records_total / parent_total,
          'currentBytesAndTypedPercent': 100 * current_bytes_and_typed / parent_total,
          'scope': ['Debug MSVC synthetic initial node-event prefix, including final independent readback.',
                    'Retained record read includes native pin, private security, identity, length and byte checks.',
                    'Time includes stage instrumentation. The residual is unassigned wall time, including instrumentation and integer rounding.',
                    'No variance control, no full recovery or product acceptance.']}
with (root / 'measurement.json').open('x', encoding='utf-8', newline='\n') as handle:
    json.dump(result, handle, indent=2, ensure_ascii=False)
    handle.write('\n')
print(json.dumps(result))


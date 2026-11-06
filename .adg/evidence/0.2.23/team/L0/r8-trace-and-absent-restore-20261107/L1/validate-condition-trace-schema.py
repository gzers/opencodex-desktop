"""Strictly validate actual Rust preview_conditions output; no renderer fixtures.
The only dependency is jsonschema 4.25.1 in a task-owned tool environment.
"""
import copy
import hashlib
import importlib.metadata
import json
from pathlib import Path
import sys
from jsonschema import Draft7Validator

root = Path(__file__).resolve().parent
log = root / 'condition-trace-cases-schema-final.log'
lines = log.read_text().splitlines()
def emitted(marker):
    return [json.loads(line[len(marker) + 1:]) for line in lines if line.startswith(marker + ' ')]
schema, = emitted('CONDITION_PREVIEW_SCHEMA')
error_schema, = emitted('CONDITION_PREVIEW_ERROR_SCHEMA')
cases = emitted('CONDITION_PREVIEW_CASE')
assert len(cases) == 14
schema['$comment'] = ('Generated from schemars::schema_for!(ConditionPreview) by '
    'commands/model_config_v2_trace_tests.rs::wire_output_schema. Only the existing '
    'FieldOriginV2 condition_id/template_id schema property/required names are corrected '
    'to the actual unchanged serde conditionId/templateId wire. No reason enum or '
    'oneOf variant, reference, or validation rule is removed or broadened.')
Draft7Validator.check_schema(schema)
Draft7Validator.check_schema(error_schema)
validator = Draft7Validator(schema)
error_validator = Draft7Validator(error_schema)
rows = 0
for case in cases:
    if case['error'] is not None:
        assert case['result'] is None
        error_validator.validate(case['error'])
        continue
    result = case['result']
    validator.validate(result)
    assert result['dtoVersion'] == 2 and result['previewOnly'] is True
    assert [r['rawModelId'] for r in result['rows']] == case['request']['rawModelIds']
    for row in result['rows']:
        rows += 1
        assert row['matchingView'] == row['rawModelId']  # no namespace in this pure request
        assert row['familyMatched'] == row['family']['matched']
        assert len(row['conditions']) == len(case['request']['content']['conditionOverrides'])
        applied = [c['conditionId'] for c in row['conditions'] if c['applied']]
        assert applied == ([] if row['matchedConditionId'] is None else [row['matchedConditionId']])
        for condition, source in zip(row['conditions'], case['request']['content']['conditionOverrides']):
            assert condition['conditionId'] == source['id']
            assert len(condition['atoms']) == len(source['selector']['all'])
            if not condition['applicable']:
                assert condition['matched'] is None and condition['applied'] is False
                assert all(atom['matched'] is None for atom in condition['atoms'])
            if any(pre['matched'] is False for pre in condition['preconditions']):
                assert all(atom['matched'] is None for atom in condition['atoms'])

valid = next(c['result'] for c in cases if c['case'] == 'numeric_joiners')
negative_checks = []
def rejected(label, value, check=validator):
    assert list(check.iter_errors(value)), label
    negative_checks.append(label)
unknown_reason = copy.deepcopy(valid)
unknown_reason['rows'][0]['conditions'][0]['atoms'][0]['reasonCode'] = 'renderer_invented_success'
rejected('unknown_reason_enum_rejected', unknown_reason)
invalid_skip = copy.deepcopy(valid)
invalid_skip['rows'][0]['conditions'][0]['atoms'][0]['matched'] = 'not_evaluated'
rejected('non_boolean_non_null_match_rejected', invalid_skip)
snake_origin = copy.deepcopy(valid)
origin = next(v for v in snake_origin['rows'][0]['fieldOrigins'].values() if v['kind'] == 'template_condition')
origin['condition_id'] = origin.pop('conditionId')
rejected('old_schemars_snake_origin_rejected', snake_origin)
extra_origin = copy.deepcopy(valid)
origin = next(v for v in extra_origin['rows'][0]['fieldOrigins'].values() if v['kind'] == 'template_condition')
origin['templateId'] = 'not-authorized-for-this-variant'
rejected('origin_oneof_extra_field_rejected', extra_origin)
missing_view = copy.deepcopy(valid)
del missing_view['rows'][0]['matchingView']
rejected('required_real_matching_view_rejected_if_missing', missing_view)
error = copy.deepcopy(next(c['error'] for c in cases if c['error'] is not None))
error['issues'][0]['conditionIds'] = [42]
rejected('invalid_error_condition_identity_type_rejected', error, error_validator)

for name, value in [('condition-preview.trace.schema.json', schema),
                    ('condition-preview.error.schema.json', error_schema),
                    ('condition-preview.real-cases.json', cases)]:
    (root / name).write_text(json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + '\n')
summary = {'validator': 'jsonschema', 'version': importlib.metadata.version('jsonschema'),
    'draft': 'draft-07', 'schema_check': 'passed', 'real_function_cases': len(cases),
    'valid_results': sum(c['result'] is not None for c in cases),
    'valid_errors': sum(c['error'] is not None for c in cases), 'rows': rows,
    'negative_assertions': negative_checks, 'cases': [c['case'] for c in cases],
    'source': 'actual preview_conditions Rust test output, no renderer declarations',
    'sha256': {name: hashlib.sha256((root / name).read_bytes()).hexdigest() for name in
        ['condition-preview.trace.schema.json', 'condition-preview.error.schema.json', 'condition-preview.real-cases.json']}}
(root / 'condition-preview.schema-validation.json').write_text(json.dumps(summary, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(summary, ensure_ascii=False, indent=2))

import assert from 'node:assert/strict'

// Expectations are fixture-owned input, captured before opening the sync UI.
// Never derive the expected set from the selection being verified.
export function assertFirstScopeSelection(scope, expected) {
  for (const key of ['channelIds', 'templateIds']) {
    assert(Array.isArray(expected[key]) && expected[key].length > 0, 'First-scope fixture must include nonempty ' + key)
    assert(expected[key].every(id => typeof id === 'string' && id.length > 0), 'Invalid expected scope identity')
    assert.equal(new Set(expected[key]).size, expected[key].length, 'Expected scope identities must be unique')
    assert(Array.isArray(scope[key]), 'Missing saved scope identities: ' + key)
    assert.equal(new Set(scope[key]).size, scope[key].length, 'Saved scope contains duplicate identities')
    assert.deepEqual([...scope[key]].sort(), [...expected[key]].sort(), 'First selection must equal complete local/cloud fixture union: ' + key)
  }
}

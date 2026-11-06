// Build read-only cloud catalogue input from two known synthetic native exports.
// This never connects to a server, opens the app, or derives IDs from saved scope.
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'

const [channelFile, templateFile, output] = process.argv.slice(2)
assert(channelFile && templateFile && output, 'usage: cloud-fixture.mjs CHANNEL_EXPORT TEMPLATE_EXPORT NEW_OUTPUT')
const bytes = value => Buffer.from(JSON.stringify(value))
const digest = value => 'sha256:' + createHash('sha256').update(value).digest('hex')
async function load(path, expectedDigest) {
  const source = await readFile(path), envelope = JSON.parse(source)
  assert.equal(digest(source), expectedDigest, 'exact reviewed synthetic export required')
  assert.equal(envelope.format, 'OCXDCONF'); assert.equal(envelope.formatVersion, 3)
  assert.equal(envelope.payload.kind, 'plain')
  const documentBytes = Buffer.from(envelope.payload.document_b64, 'base64')
  assert.equal(digest(documentBytes), envelope.manifest.documentDigest)
  const document = JSON.parse(documentBytes)
  assert.equal(digest(bytes(document.models.payload)), document.models.digest)
  return { sourceDigest: digest(source), envelope, document }
}
const channel = await load(channelFile, 'sha256:06f235a32caf7745547aae8cc16bffcb87348eedfb97d493dc575d89156240bb')
const template = await load(templateFile, 'sha256:c97b170d301bd9caea87edeecfdb4e41cc1bbfb9b6881621751d9169f9011387')
assert.equal(channel.envelope.manifest.id, '00f5db9f-5684-4097-887d-5db67e45f12f', 'reviewed synthetic channel export required')
assert.equal(template.envelope.manifest.id, 'e64239ba-42dc-490f-8908-ae476d6fefbe', 'reviewed synthetic template export required')
assert.equal(channel.document.models.payload.policy.channels.length, 1)
assert.equal(template.document.models.payload.templates.length, 1)
const identities = {
  initial: { channelId: '7e89780b-058f-4d51-91ce-925376425401', templateId: '7e89780b-058f-4d51-91ce-925376425402', revisionId: '7e89780b-058f-4d51-91ce-925376425403', nodeId: '7e89780b-058f-4d51-91ce-925376425404' },
  later: { channelId: '7e89780b-058f-4d51-91ce-925376425411', templateId: '7e89780b-058f-4d51-91ce-925376425412', revisionId: '7e89780b-058f-4d51-91ce-925376425413', nodeId: '7e89780b-058f-4d51-91ce-925376425414' },
}
function makeChannel(id, label) {
  const row = structuredClone(channel.document.models.payload.policy.channels[0])
  row.id = id.channelId; row.label = label
  row.connection.nodes = [{ id: id.nodeId, name: 'Disabled synthetic node', url: 'https://cloud-fixture.invalid/v1' }]
  // Replace public provider metadata with a small, credential-free disabled fixture.
  row.connection.provider = { adapter: 'openai-chat', authMode: 'key', baseUrl: 'https://cloud-fixture.invalid/v1', disabled: true, models: ['fixture-alpha', 'fixture-beta', 'fixture-gamma'], newModelPolicy: 'off', selectedModels: [] }
  return row
}
function makeTemplate(id, label) {
  const row = structuredClone(template.document.models.payload.templates[0])
  row.id = id.templateId; row.label = label; row.defaultRevision = id.revisionId
  assert.equal(row.revisions.length, 1)
  row.revisions[0].id = id.revisionId
  // savedLabel/content/digest stay together; no fabricated saved content version.
  return row
}
await mkdir(output, { recursive: false })
const manifest = { purpose: 'Synthetic catalogue fixtures only; native acceptance not yet run', sourceDigests: [channel.sourceDigest, template.sourceDigest], identities, stages: {} }
for (const [stage, names] of [['initial', ['initial']], ['later', ['initial', 'later']]]) {
  const document = structuredClone(channel.document), model = document.models
  document.version = 4
  document.channelKeys = {}; delete document.channelAuthorizations
  document.manager = { preferences: null, extensions: null, webdav: [], accounts: [] }
  document.webdavPasswords = {}; document.assets = { skills: [], mcps: [] }; document.contentVersions = []
  model.payload.templates = names.map(name => makeTemplate(identities[name], 'Cloud-only ' + name + ' template'))
  model.payload.policy = { templateRefs: null, channelOrder: null, channels: names.map(name => makeChannel(identities[name], 'Cloud-only ' + name + ' channel')) }
  model.digest = digest(bytes(model.payload))
  const documentBytes = bytes(document), envelope = structuredClone(channel.envelope)
  envelope.manifest.id = stage === 'initial' ? '7e89780b-058f-4d51-91ce-925376425420' : '7e89780b-058f-4d51-91ce-925376425421'
  envelope.manifest.sections = ['models']; envelope.manifest.secretObjects = 0
  envelope.manifest.documentDigest = digest(documentBytes)
  envelope.payload.document_b64 = documentBytes.toString('base64')
  const envelopeBytes = bytes(envelope), envelopeDigest = digest(envelopeBytes), filename = 'config-v1-' + envelopeDigest.slice(7) + '.ocxd'
  const pointer = { format: 'OCXD_CONFIG_POINTER', version: 1, operationId: envelope.manifest.id, digest: envelopeDigest }
  const pointerBytes = bytes(pointer), directory = join(output, stage)
  await mkdir(directory)
  await writeFile(join(directory, filename), envelopeBytes, { flag: 'wx' })
  await writeFile(join(directory, 'config-v1-latest.json'), pointerBytes, { flag: 'wx' })
  await writeFile(join(directory, 'etags.json'), bytes({ [filename]: '"' + envelopeDigest.slice(7) + '"', 'config-v1-latest.json': '"' + digest(pointerBytes).slice(7) + '"' }), { flag: 'wx' })
  manifest.stages[stage] = { filename, digest: envelopeDigest, channelIds: names.map(name => identities[name].channelId), templateIds: names.map(name => identities[name].templateId), secretObjects: 0, contentVersions: 0 }
}
await writeFile(join(output, 'fixture-manifest.json'), JSON.stringify(manifest, null, 2) + '\n', { flag: 'wx' })
console.log(JSON.stringify(manifest, null, 2))

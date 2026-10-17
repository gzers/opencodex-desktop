// Publish a complete diagnostic marker once. Readers cannot see a partial JSON
// file, and another writer cannot replace an already published observation.
import { randomUUID } from 'node:crypto'
import { link, unlink, writeFile } from 'node:fs/promises'

export async function writeFreshObservationJson(path, value) {
  const temporary = `${path}.${randomUUID()}.tmp`
  await writeFile(temporary, JSON.stringify(value, null, 2) + '\n', { flag: 'wx' })
  try {
    await link(temporary, path)
  } finally {
    await unlink(temporary)
  }
}

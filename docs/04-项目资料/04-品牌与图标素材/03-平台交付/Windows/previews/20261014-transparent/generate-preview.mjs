// Unapproved Windows visual preview; never modifies macOS or production assets.
import { createRequire } from 'node:module'
import { readFile, writeFile, mkdir } from 'node:fs/promises'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createHash } from 'node:crypto'

const require = createRequire('C:/Users/15119/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/package.json')
const sharp = require('sharp')
const root = dirname(fileURLToPath(import.meta.url))
const brand = join(root, '../../../..')
const source = join(brand, '03-平台交付/macOS/AppIcon/AppIcon.svg')
let svg = await readFile(source, 'utf8')
const background = /  <defs>[\s\S]*?id="app-background"[\s\S]*?<\/defs>\s*<rect width="1024" height="1024" rx="228" fill="url\(#app-background\)"\/>/
if (!background.test(svg)) throw Error('Expected the documented macOS background exactly once')
svg = svg.replace(background, '')

function bounds(data, width, height, threshold = 16) {
  let left = width, top = height, right = 0, bottom = 0
  for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
    if (data[(y * width + x) * 4 + 3] > threshold) {
      left = Math.min(left, x); top = Math.min(top, y)
      right = Math.max(right, x + 1); bottom = Math.max(bottom, y + 1)
    }
  }
  if (right <= left || bottom <= top) throw Error('No artwork')
  return [left, top, right, bottom]
}
const base = await sharp(Buffer.from(svg)).resize(1024, 1024).ensureAlpha().raw().toBuffer()
const bbox = bounds(base, 1024, 1024)
const center = [(bbox[0] + bbox[2]) / 2, (bbox[1] + bbox[3]) / 2]
const edge = Math.max(bbox[2] - bbox[0], bbox[3] - bbox[1])
const observations = []

async function variant(name, coverage, mono) {
  const span = edge / coverage
  const viewBox = [center[0] - span / 2, center[1] - span / 2, span, span].join(' ')
  let vector = svg.replace('viewBox="0 0 1024 1024"', `viewBox="${viewBox}"`)
  if (mono) vector = vector.replace('fill="url(#codex-gradient)"', `fill="${mono}"`)
  vector = vector.replace(/<title id="title">[^<]*<\/title>/,
    `<title id="title">OpenCodex Windows ${name} preview — not approved</title>`)
  await writeFile(join(root, name + '.svg'), vector)
  const sizes = name.startsWith('tray') ? [16, 20, 24, 32, 40, 48, 64] : [16, 24, 32, 48, 64, 128, 256, 512]
  const frames = []
  await mkdir(join(root, name), { recursive: true })
  for (const size of sizes) {
    let png = await sharp(Buffer.from(vector)).resize(size, size).png().toBuffer()
    const raw = await sharp(png).ensureAlpha().raw().toBuffer()
    if (mono) {
      // Keep a single foreground colour; only alpha carries edge antialiasing.
      const colour = Buffer.from(mono.slice(1), 'hex')
      for (let offset = 0; offset < raw.length; offset += 4) {
        raw[offset] = colour[0]; raw[offset + 1] = colour[1]; raw[offset + 2] = colour[2]
      }
      png = await sharp(raw, { raw: { width: size, height: size, channels: 4 } }).png().toBuffer()
    }
    const actual = bounds(raw, size, size)
    const margin = Math.floor(size * 0.03)
    if (raw[3] !== 0 || raw[(size - 1) * 4 + 3] !== 0
        || raw[((size - 1) * size) * 4 + 3] !== 0
        || raw[(size * size - 1) * 4 + 3] !== 0) throw Error('Opaque corner')
    if (actual[0] < margin || actual[1] < margin) throw Error('Artwork clipped')
    await writeFile(join(root, name, `${size}.png`), png)
    frames.push({ size, png, bbox: actual, sha256: createHash('sha256').update(png).digest('hex') })
  }
  if (name.startsWith('app')) {
    const icons = frames.filter(f => f.size <= 256)
    const header = Buffer.alloc(6 + icons.length * 16)
    header.writeUInt16LE(1, 2); header.writeUInt16LE(icons.length, 4)
    let offset = header.length
    icons.forEach((frame, index) => {
      const start = 6 + index * 16
      header[start] = frame.size === 256 ? 0 : frame.size
      header[start + 1] = header[start]
      header.writeUInt16LE(1, start + 4); header.writeUInt16LE(32, start + 6)
      header.writeUInt32LE(frame.png.length, start + 8); header.writeUInt32LE(offset, start + 12)
      offset += frame.png.length
    })
    await writeFile(join(root, name + '.ico'), Buffer.concat([header, ...icons.map(f => f.png)]))
  }
  observations.push({ name, requestedCoverage: coverage, monochrome: mono ?? null,
    frames: frames.map(({ png, ...metadata }) => metadata) })
}
await variant('app-85', 0.85)
await variant('app-90', 0.90)
await variant('tray-light', 0.90, '#FFFFFF')
await variant('tray-dark', 0.90, '#1A2948')
await writeFile(join(root, 'generation-report.json'), JSON.stringify({
  status: 'unapproved-preview', source, sourceSha256: createHash('sha256').update(await readFile(source)).digest('hex'),
  tools: sharp.versions, sourceArtworkBounds: bbox,
  appCandidateAppliedToTest: 'app-90', foregroundOrNativeInputIssuedByGenerator: false,
  macosAssetsModified: false, productionAssetsModified: false, variants: observations,
}, null, 2) + '\n')
console.log(JSON.stringify({ status: 'preview-generated', variants: observations.map(v => v.name) }))

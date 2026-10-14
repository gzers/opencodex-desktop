// Export the approved Windows vectors. The optional code target receives only
// consumable assets; macOS/default icons are never changed.
import { createRequire } from 'node:module'
import { readFile, writeFile, mkdir } from 'node:fs/promises'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createHash } from 'node:crypto'

const require = createRequire(process.env.OPENCODEX_SHARP_PACKAGE
  ?? join(dirname(fileURLToPath(import.meta.url)), 'package.json'))
const sharp = require('sharp')
const root = dirname(fileURLToPath(import.meta.url))
const codeOutput = process.argv[2] ? resolve(process.argv[2]) : null
const hash = value => createHash('sha256').update(value).digest('hex')
const manifest = []
const inputs = []
async function save(path, content, codeName) {
  await mkdir(dirname(path), { recursive: true })
  await writeFile(path, content)
  if (codeOutput && codeName) {
    await mkdir(codeOutput, { recursive: true })
    await writeFile(join(codeOutput, codeName), content)
  }
  manifest.push({ path: path.slice(root.length + 1).replaceAll('\\', '/'),
    bytes: content.length, sha256: hash(content), codeName })
}
function ico(frames) {
  const header = Buffer.alloc(6 + frames.length * 16)
  header.writeUInt16LE(1, 2); header.writeUInt16LE(frames.length, 4)
  let offset = header.length
  frames.forEach(({ size, bytes }, index) => {
    const at = 6 + index * 16
    header[at] = size === 256 ? 0 : size; header[at + 1] = header[at]
    header.writeUInt16LE(1, at + 4); header.writeUInt16LE(32, at + 6)
    header.writeUInt32LE(bytes.length, at + 8); header.writeUInt32LE(offset, at + 12)
    offset += bytes.length
  })
  return Buffer.concat([header, ...frames.map(frame => frame.bytes)])
}
for (const name of ['AppIcon', 'tray-light', 'tray-dark']) {
  const isApp = name === 'AppIcon'
  const folder = join(root, isApp ? 'AppIcon' : 'Tray')
  const vector = await readFile(join(folder, name + '.svg'))
  inputs.push({ path: (isApp ? 'AppIcon/' : 'Tray/') + name + '.svg', sha256: hash(vector) })
  const sizes = isApp ? [16, 24, 32, 48, 64, 128, 256, 512] : [16, 20, 24, 32, 40, 48, 64]
  const frames = []
  for (const size of sizes) {
    // At 16px, reserve a full transparent pixel instead of letting Lanczos
    // antialiasing touch the canvas. Larger sizes retain the approved 90% viewBox.
    const rasterSize = size === 16 ? 14 : size
    let raster = sharp(vector).resize(rasterSize, rasterSize)
    if (size === 16) raster = raster.extend({ top: 1, bottom: 1, left: 1, right: 1,
      background: { r: 0, g: 0, b: 0, alpha: 0 } })
    let bytes = await raster.ensureAlpha().png().toBuffer()
    if (!isApp) {
      const rgba = await sharp(bytes).ensureAlpha().raw().toBuffer()
      const color = name === 'tray-light' ? [255, 255, 255] : [26, 41, 72]
      for (let at = 0; at < rgba.length; at += 4) {
        rgba[at] = color[0]; rgba[at + 1] = color[1]; rgba[at + 2] = color[2]
      }
      bytes = await sharp(rgba, { raw: { width: size, height: size, channels: 4 } }).png().toBuffer()
    }
    frames.push({ size, bytes })
    await save(isApp ? join(folder, 'png', `${size}.png`) : join(folder, `${name}-${size}.png`),
      bytes, isApp ? `${size}x${size}.png` : `${name}-${size}.png`)
    if (!isApp && size === 32) await save(join(folder, `${name}@2x.png`), bytes)
  }
  await save(join(folder, name + '.ico'),
    ico(frames.filter(frame => isApp ? frame.size <= 256 : [16, 24, 32].includes(frame.size))),
    isApp ? 'icon.ico' : null)
}
await writeFile(join(root, 'assets-20261014.json'), JSON.stringify({
  approval: 'User approved the 90% transparent app and monochrome tray preview on 2026-10-14',
  largeSubjectCoverage: 0.90, small16PixelSafetyMargin: 1,
  tools: sharp.versions, inputs, artifacts: manifest,
}, null, 2) + '\n')
console.log(JSON.stringify({ exported: manifest.length, codeOutput, macosAssetsModified: false }))

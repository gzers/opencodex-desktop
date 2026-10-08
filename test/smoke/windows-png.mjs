// Decode CDP's 8-bit RGB/RGBA PNG and measure a quiet, known stripe region.
// This tests composed pixels rather than treating a CSS blur declaration as proof.
import assert from 'node:assert/strict'
import { inflateSync } from 'node:zlib'
export function pngStripeContrast(bytes, rectangle, scale) {
  const chunks = []
  let width, height, bpp
  for (let offset = 8; offset < bytes.length;) {
    const size = bytes.readUInt32BE(offset), type = bytes.toString('ascii', offset + 4, offset + 8)
    const chunk = bytes.subarray(offset + 8, offset + 8 + size)
    if (type === 'IHDR') {
      width = chunk.readUInt32BE(0); height = chunk.readUInt32BE(4)
      assert.equal(chunk[8], 8); assert([2, 6].includes(chunk[9])); assert.equal(chunk[12], 0)
      bpp = chunk[9] === 6 ? 4 : 3
    }
    if (type === 'IDAT') chunks.push(chunk)
    offset += size + 12
  }
  const raw = inflateSync(Buffer.concat(chunks)), stride = width * bpp
  const pixels = Buffer.alloc(height * stride)
  const paeth = (a, b, c) => { const p = a + b - c, x = Math.abs(p - a), y = Math.abs(p - b), z = Math.abs(p - c); return x <= y && x <= z ? a : y <= z ? b : c }
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)]
    assert(filter <= 4)
    for (let x = 0; x < stride; x++) {
      const a = x >= bpp ? pixels[y * stride + x - bpp] : 0
      const b = y ? pixels[(y - 1) * stride + x] : 0
      const c = y && x >= bpp ? pixels[(y - 1) * stride + x - bpp] : 0
      const predictor = [0, a, b, Math.floor((a + b) / 2), paeth(a, b, c)][filter]
      pixels[y * stride + x] = (raw[y * (stride + 1) + x + 1] + predictor) & 255
    }
  }
  // Right-hand side of the menu below its first option, away from glyphs/borders.
  const x0 = Math.floor((rectangle.x + rectangle.width * .72) * scale)
  const x1 = Math.floor((rectangle.x + rectangle.width * .9) * scale)
  const y0 = Math.floor((rectangle.y + rectangle.height * .5) * scale)
  const y1 = Math.floor((rectangle.y + rectangle.height * .8) * scale)
  assert(x0 >= 0 && x1 < width && y0 >= 0 && y1 < height)
  let sum = 0, count = 0
  for (let y = y0; y < y1; y++) for (let x = x0 + 1; x < x1; x++) {
    const offset = y * stride + x * bpp
    sum += Math.abs(pixels[offset] - pixels[offset - bpp])
    count++
  }
  assert(count > 0)
  return sum / count
}

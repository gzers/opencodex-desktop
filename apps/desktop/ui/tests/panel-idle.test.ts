import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'
const script = readFileSync('../tauri/assets/panel-idle.js', 'utf8')
function harness() {
  const doc = document.implementation.createHTMLDocument()
  const view: { __ocxdTryReclaim?: (generation: number) => void } = {}
  const location = { pathname: '/web', hash: '#providers/detail', href: '' }
  new Function('window', 'document', 'location', 'scrollY', script)(view, doc, location, 120)
  return { doc, view, location }
}
describe('idle panel preserves user work', () => {
  it('offers clean view reclamation with location, scroll and generation', () => {
    const h = harness(); h.view.__ocxdTryReclaim!(7)
    const url = new URL(h.location.href)
    expect(url.hostname).toBe('reclaim')
    expect(url.searchParams.get('generation')).toBe('7')
    expect(url.searchParams.get('fragment')).toBe('providers/detail')
    expect(url.searchParams.get('path')).toBe('/web')
    expect(url.searchParams.get('scroll')).toBe('120')
  })
  it.each(['input', 'change'])('keeps a view after %s, even after a long idle period', event => {
    const h = harness()
    h.doc.dispatchEvent(new Event(event)); h.view.__ocxdTryReclaim!(7)
    expect(h.location.href).toBe('')
  })
})

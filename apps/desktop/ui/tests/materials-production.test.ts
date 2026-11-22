// @vitest-environment node
import { readdirSync, readFileSync } from 'node:fs'
import { join, relative, resolve } from 'node:path'
import { parse } from '@vue/compiler-sfc'
import { transform } from 'lightningcss'
import { expect, it } from 'vitest'

const root = resolve('src')
function sources(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const path = join(directory, entry.name)
    return entry.isDirectory() ? sources(path) : /\.(css|vue)$/.test(entry.name) ? [path] : []
  })
}

it('preserves a standard backdrop filter through production minification in every material rule', () => {
  const failures: { file: string; selector: string }[] = []
  let filteredRules = 0
  for (const path of sources(root)) {
    const source = readFileSync(path, 'utf8')
    const styles = path.endsWith('.vue') ? parse(source, { filename: path }).descriptor.styles.map(style => style.content) : [source]
    for (const css of styles) {
      // Vite uses this locked minifier. Identical prefixed/unprefixed declarations
      // can become only the last spelling, even though source-level checks pass.
      const built = transform({ filename: path, code: Buffer.from(css), minify: true }).code.toString()
      for (const [, selector, body] of built.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
        const standard = /(?:^|;)backdrop-filter:/.test(body)
        if (standard) filteredRules++
        if (/(?:^|;)-webkit-backdrop-filter:/.test(body) && !standard) failures.push({ file: relative(root, path), selector })
      }
    }
  }
  expect(filteredRules).toBeGreaterThan(15)
  expect(failures).toEqual([])
})

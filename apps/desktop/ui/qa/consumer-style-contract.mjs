import { parse } from '@vue/compiler-sfc'
import postcss from 'postcss'

// Public recipe/layout owns visual metrics. Consumers may size their placement,
// but cannot overwrite the control/content typography and inset contract.
const protectedProperties = /^(?:font(?:-.+)?|padding(?:-.+)?|height|min-height|max-height|line-height|border-radius|transform|vertical-align)$/
export function consumerStyleViolations(source, filename = 'consumer.vue') {
 const { descriptor } = parse(source, { filename })
 const buttonClasses = new Set([...descriptor.template?.content.matchAll(/<UiButton\b[^>]*\bclass="([^"]*)"/g) ?? []].flatMap(m => m[1].split(/\s+/).filter(Boolean)))
 const violations = []
 for (const block of descriptor.styles) {
  const sheet = postcss.parse(block.content, { from: filename })
  sheet.walkRules(rule => {
   for (const selector of rule.selector.split(',')) {
    // Do not attribute descendant icons, labels or nested controls to the root.
    const target = selector.trim().replace(/:deep\(([^)]+)\)/g, '$1').split(/[>+~\s]+/).at(-1) ?? ''
    const classes = [...target.matchAll(/\.([\w-]+)/g)].map(m => m[1])
    const controls = classes.some(c => c === 'btn' || c === 'btn-content' || buttonClasses.has(c))
    if (!controls) continue
    rule.walkDecls(decl => {
     if (protectedProperties.test(decl.prop)) violations.push({ file: filename, selector: selector.trim(), property: decl.prop })
    })
   }
  })
 }
 return violations
}

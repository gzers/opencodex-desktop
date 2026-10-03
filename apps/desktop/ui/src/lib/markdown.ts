// 扩展条目详情的 Markdown 渲染：**先整段转义再套标记**，正文里的 HTML 不作为标签执行；
// 链接只保留文字、不生成 `href`（详情是只读展示，不提供外链跳转）。

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

function inline(value: string): string {
  return escapeHtml(value)
    // 行内代码先处理，避免其中的 `**` 被当成加粗。
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    // 链接只留文字：`[文字](地址)` → `文字`。
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
}

/**
 * 渲染受限 Markdown（标题 / 有序与无序列表 / 引用 / 围栏代码 / 加粗 / 行内代码）。
 * 输出始终是转义过的 HTML 片段。
 */
export function renderMarkdown(source: string): string {
  const lines = source.replace(/\r\n?/g, '\n').split('\n')
  const html: string[] = []
  let index = 0
  let listKind: 'ul' | 'ol' | null = null

  const closeList = () => {
    if (listKind) {
      html.push(`</${listKind}>`)
      listKind = null
    }
  }

  while (index < lines.length) {
    const line = lines[index]

    const fence = line.match(/^\s*```(.*)$/)
    if (fence) {
      closeList()
      const body: string[] = []
      index += 1
      while (index < lines.length && !/^\s*```/.test(lines[index])) {
        body.push(lines[index])
        index += 1
      }
      index += 1
      html.push(`<pre><code>${escapeHtml(body.join('\n'))}</code></pre>`)
      continue
    }

    if (!line.trim()) {
      closeList()
      index += 1
      continue
    }

    const heading = line.match(/^(#{1,6})\s+(.*)$/)
    if (heading) {
      closeList()
      const level = heading[1].length
      html.push(`<h${level}>${inline(heading[2].trim())}</h${level}>`)
      index += 1
      continue
    }

    if (/^\s*>\s?/.test(line)) {
      closeList()
      const body: string[] = []
      while (index < lines.length && /^\s*>\s?/.test(lines[index])) {
        body.push(lines[index].replace(/^\s*>\s?/, ''))
        index += 1
      }
      html.push(`<blockquote>${inline(body.join(' '))}</blockquote>`)
      continue
    }

    const ordered = line.match(/^\s*\d+[.)]\s+(.*)$/)
    if (ordered) {
      if (listKind !== 'ol') {
        closeList()
        html.push('<ol>')
        listKind = 'ol'
      }
      html.push(`<li>${inline(ordered[1])}</li>`)
      index += 1
      continue
    }

    const bullet = line.match(/^\s*[-*+]\s+(.*)$/)
    if (bullet) {
      if (listKind !== 'ul') {
        closeList()
        html.push('<ul>')
        listKind = 'ul'
      }
      html.push(`<li>${inline(bullet[1])}</li>`)
      index += 1
      continue
    }

    closeList()
    html.push(`<p>${inline(line.trim())}</p>`)
    index += 1
  }

  closeList()
  return html.join('')
}

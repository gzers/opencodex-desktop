export interface TreeTableNode {
  id: string
  label: string
  children?: readonly TreeTableNode[]
  selectable?: boolean
  selected?: boolean | "mixed"
  disabled?: boolean
}

export interface TreeTableColumn {
  key: string
  label: string
}

export interface TreeTableRow {
  node: TreeTableNode
  depth: number
  parentId?: string
}

/** Visible rows are a projection: collapse never changes selection or payload. */
export function treeTableRows(nodes: readonly TreeTableNode[], expanded: ReadonlySet<string>): TreeTableRow[] {
  const result: TreeTableRow[] = []
  const visit = (items: readonly TreeTableNode[], depth: number, parentId?: string) => {
    for (const node of items) {
      result.push({ node, depth, parentId })
      if (node.children?.length && expanded.has(node.id)) visit(node.children, depth + 1, node.id)
    }
  }
  visit(nodes, 0)
  return result
}

export function initialTreeExpansion(nodes: readonly TreeTableNode[], depth: number): string[] {
  const expanded: string[] = []
  const visit = (items: readonly TreeTableNode[], level: number) => {
    if (level >= depth) return
    for (const node of items) {
      if (node.children?.length) {
        expanded.push(node.id)
        visit(node.children, level + 1)
      }
    }
  }
  visit(nodes, 0)
  return expanded
}

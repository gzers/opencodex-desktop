/** Reorder only visible, eligible slots; absent model stubs keep their positions. */
export function reorderSlots(order: readonly string[], eligible: readonly string[], id: string, targetId: string): string[] {
  const allowed = new Set(eligible)
  const visible = order.filter(value => allowed.has(value))
  const from = visible.indexOf(id), to = visible.indexOf(targetId)
  if (from < 0 || to < 0 || from === to) return [...order]
  visible.splice(to, 0, visible.splice(from, 1)[0]!)
  let cursor = 0
  return order.map(value => allowed.has(value) ? visible[cursor++]! : value)
}

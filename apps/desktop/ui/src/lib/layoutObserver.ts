/** Layout observers must not resize their observed tree inside ResizeObserver
 * delivery. Coalesce into the next animation frame, retaining initial explicit
 * measurements at each consumer and cancelling work when its owner unmounts. */
export function layoutObserver(update: () => void): Pick<ResizeObserver, 'observe' | 'disconnect'> | undefined {
  if (typeof ResizeObserver === 'undefined') return undefined
  let frame: number | undefined, disposed = false
  const request = typeof requestAnimationFrame === 'function' ? requestAnimationFrame : (fn: FrameRequestCallback) => window.setTimeout(() => fn(performance.now()), 16)
  const cancel = typeof cancelAnimationFrame === 'function' ? cancelAnimationFrame : window.clearTimeout
  const observer = new ResizeObserver(() => {
    if (disposed || frame !== undefined) return
    frame = request(() => { frame = undefined; if (!disposed) update() })
  })
  return {
    observe: (...args) => observer.observe(...args),
    disconnect() { disposed = true; observer.disconnect(); if (frame !== undefined) cancel(frame); frame = undefined },
  }
}

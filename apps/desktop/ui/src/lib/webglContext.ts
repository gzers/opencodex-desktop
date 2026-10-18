// Explicitly released contexts must not be treated as device failures by the
// document's capture listener, including an asynchronously delivered loss event.
const releasedCanvases = new WeakSet<EventTarget>()

export function isReleasedWebglCanvas(target: EventTarget | null): boolean {
  return target !== null && releasedCanvases.has(target)
}

export function releaseWebglContext(canvas: HTMLCanvasElement, gl: WebGLRenderingContext): void {
  if (releasedCanvases.has(canvas)) return
  releasedCanvases.add(canvas)
  gl.getExtension('WEBGL_lose_context')?.loseContext()
}

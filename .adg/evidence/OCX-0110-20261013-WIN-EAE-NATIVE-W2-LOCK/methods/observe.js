(() => {
  const scene = window.__ocxEaeW2Lock
  if (!scene) throw Error('Original scene absent')
  return { atUtc: new Date().toISOString(), atMs: performance.now(), phaseIndex: scene.phaseIndex,
    preparedPhase: scene.preparedPhase, operation: scene.operation ? structuredClone(scene.operation) : null,
    scope: 'read-only actual pending native command witness; no invocation, route, focus or input' }
})()

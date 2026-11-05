import { readFile } from 'node:fs/promises'

export function handshakeWaitSeconds(value = '3600') {
  if (!/^[0-9]+$/.test(value) || Number(value) < 1 || Number(value) > 43200) {
    throw Error('Administrator handshake wait must be 1..43200 seconds')
  }
  return Number(value)
}

// Accept historical Windows PowerShell 5.1 UTF-8 output too. A partial write
// may be retried; permissions and other IO errors must remain visible.
export async function readCollectorStart(file) {
  let raw
  try { raw = await readFile(file, 'utf8') }
  catch (error) { if (error.code === 'ENOENT') return null; throw error }
  try { return JSON.parse(raw.replace(/^﻿/, '')) }
  catch (error) { if (error instanceof SyntaxError) return null; throw error }
}

export function assertCaptureOverlap(started, now = Date.now()) {
  const at = Date.parse(started.startedAtUtc)
  if (!started.recorderStarted || !Number.isFinite(at) ||
      !/^ocx-post021-gpu-[a-f0-9]{32}$/.test(started.instanceName) ||
      started.durationSeconds !== 45) throw Error('Invalid successful 45s recording handshake')
  if (now < at || now - at >= 10000) throw Error('Recording cannot cover the complete 30s foreground sample; preserve it as diagnostic only')
}

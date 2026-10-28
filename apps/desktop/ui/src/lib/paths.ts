/**
 * Presentation only. Callers retain the backend path for opens, writes and execution.
 * This is not filesystem validation and never resolves dots, links or device names.
 */
export function displayPath(value: string | null | undefined, platform?: string | null): string {
  if (!value || (platform && platform.toLowerCase() !== 'windows')) return value ?? ''
  if (!platform && !/^(?:[a-z]:[\\/]|\\\\)/i.test(value)) return value
  if (value !== value.trim() || /[\u0000-\u001f]/.test(value)) return value
  const path = value.replaceAll('/', '\\')
  let ordinary: string
  if (/^\\\\\?\\UNC\\/i.test(path)) ordinary = '\\\\' + path.slice(8)
  else if (/^\\\\\?\\[a-z]:\\/i.test(path)) ordinary = path.slice(4)
  else ordinary = path
  const drive = /^[a-z]:\\(.*)$/i.exec(ordinary)
  const share = /^\\\\([^\\]+)\\([^\\]+)(?:\\(.*))?$/.exec(ordinary)
  if (!drive && !share) return value
  const parts = drive ? drive[1]!.split('\\') : [share![1]!, share![2]!, ...(share![3] ?? '').split('\\')]
  if (parts.some(part => part === '.' || part === '..' || /[:<>"|?*]/.test(part)
    || /[. ]$/.test(part) || /^(con|prn|aux|nul|com[1-9¹²³]|lpt[1-9¹²³])(?:\.|$)/i.test(part))) return value
  return ordinary.length < 260 ? ordinary : path
}

/** A local selected filename, independent of the host rendering platform. */
export function localFileName(value: string): string {
  return value.split(/[\\/]/).at(-1) || value
}

export function localPathExample(name: string, platform?: string | null): string {
  return platform?.toLowerCase() === 'windows' ? `D:\\${name}` : `/absolute/path/to/${name}`
}

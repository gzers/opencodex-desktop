export type DoctorMode = 'readOnly'

export interface DoctorDto {
  mode: DoctorMode
  lines: string[]
  truncated: boolean
}

export async function runDoctor(network = false): Promise<DoctorDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<DoctorDto>('run_doctor', { network })
}

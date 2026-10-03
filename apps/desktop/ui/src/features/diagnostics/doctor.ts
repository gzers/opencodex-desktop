export type DoctorMode = 'read_only'

export interface DoctorDto {
  mode: DoctorMode
  lines: string[]
  truncated: boolean
}

export async function runDoctor(): Promise<DoctorDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<DoctorDto>('run_doctor')
}

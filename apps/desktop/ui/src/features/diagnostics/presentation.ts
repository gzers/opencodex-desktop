import type { DoctorDto } from '@/features/diagnostics/doctor'

export function doctorReportText(report: DoctorDto | null, shown: boolean): string {
  if (!shown) return ''
  if (!report) return ''
  return report.lines.join('\n')
}

/** Public v1 file format, independent of application and IPC models. */
export interface ExportJSONSchema {
  version: string
  exportedAt: string
  count: number
  questions: Array<{ prompt: string; answer: string; analysis: string }>
}

export interface ImportResult {
  success: number
  skipped: number
  failed: number
  errors: string[]
}

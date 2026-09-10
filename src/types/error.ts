export interface ApiError {
  code:
    | 'INVALID_ARGUMENT'
    | 'NOT_FOUND'
    | 'MISSING_REFERENCE'
    | 'RESOURCE_IN_USE'
    | 'CORRUPTED_DATA'
    | 'STORAGE_ERROR'
  message: string
  details?: unknown
}

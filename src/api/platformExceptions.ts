import { invoke } from '@tauri-apps/api/core'

// Explicit migration exceptions approved for the existing platform/maintenance
// features. Replace these wrappers when equivalent Current commands exist.
export const getOpenedUrls = () => invoke<string[]>('legacy_opened_urls')

export const purgeSyncedDeletions = () =>
  invoke<Record<string, { deleted: number }>>('legacy_purge_synced_deletions')

export const checkAndDeleteOrphans = () =>
  invoke<{ total_checked: number; orphan_records_soft_deleted: unknown[] }>(
    'legacy_check_orphan_records'
  )

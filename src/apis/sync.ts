import { invoke } from '@tauri-apps/api/core'

/**
 * 当前远程同步服务器实现不受支持。本模块只保留完全本地的数据维护操作。
 */

export async function purgeSyncedDeletions(): Promise<
  Record<string, { deleted: number }>
> {
  return invoke('purge_synced_deletions')
}

export async function checkAndDeleteOrphans(): Promise<{
  orphan_records_soft_deleted: string[]
  total_checked: number
}> {
  return invoke('check_orphan_records')
}

/**
 * 同步 API 和协议实现 - 所有表同步
 */

import { invoke } from '@tauri-apps/api/core'

// ==================== 服务器响应数据类型 ====================

/** 服务器记录通用格式（to_dict 输出） */
export interface ServerRecord {
  id: string // 记录 ID (UUID)
  table_name: string // 表名 (error_questions, subjects, srs_data, attachments, error_tags, sources)
  version: number // 版本号
  status: 'synced' | 'pending' // 同步状态
  deleted_at: number | null // 软删除时间戳
  updated_at: number // 更新时间
  data: object // 所有业务字段
}

/** 本地记录头（握手用，不含 data 负载，所有 status 的记录） */
export interface SyncRecordHeader {
  id: string
  table_name: string
  version: number
  status: 'synced' | 'pending'
  deleted_at: number | null
  updated_at: number
  created_at: number
}

/** 获取所有同步数据响应 */
interface GetAllSyncDataResponse {
  all_records: ServerRecord[]
}

/** 上传记录响应 */
interface UploadRecordResponse {
  success: boolean
  new_version?: number
  conflict?: boolean
  server_version?: number
  error?: string
}

/** 下载记录响应 */
interface DownloadRecordResponse {
  record: ServerRecord
}

// ==================== 同步状态类型 ====================

/** 握手结果 */
export interface SyncHandshakeResult {
  pull_list: string[] // 需要从服务端拉取的记录
  push_list: string[] // 可以安全推送到服务端的记录 ID
  conflicts: ConflictInfo[] // 需要用户处理的冲突
}

/** 冲突信息 */
export interface ConflictInfo {
  id: string // 记录 ID
  local_version: number // 本地版本
  server_version: number // 服务端版本
  local_deleted: boolean // 本地是否已软删除
  server_deleted: boolean // 服务端是否已软删除
}

/** 冲突解决方案 */
export type ConflictResolution = 'keep_local' | 'keep_remote'

/** 解决后的冲突记录 */
export interface ResolvedConflict {
  id: string
  resolution: ConflictResolution
}

// ==================== 进度类型 ====================

/** 同步进度状态 */
export interface SyncProgress {
  phase:
    | 'handshake'
    | 'pulling'
    | 'pushing'
    | 'resolving_conflicts'
    | 'updating'
  total: number
  current: number
  details: {
    pulled: number
    pushed: number
    conflicts_resolved: number
    failed: number
  }
}

// ==================== API 接口函数 ====================

function getBaseUrl(): string {
  return localStorage.getItem('sync_server_url') || 'http://localhost:5000'
}

/**
 * 检测服务器是否在线
 */
export async function checkServerHealth(): Promise<boolean> {
  try {
    const res = await fetch(`${getBaseUrl()}/health`)
    console.log('Server health:', res)
    if (!res.ok) return false
    const data: { status: string } = await res.json()
    return data.status === 'ok'
  } catch {
    return false
  }
}

/**
 * 验证 auth_key
 */
export async function validateAuthKey(auth_key: string): Promise<boolean> {
  const res = await fetch(`${getBaseUrl()}/api/auth/validate`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ auth_key })
  })
  const data = await res.json()
  return data.valid === true
}

/**
 * 获取当前用户的所有同步数据（服务端所有记录）
 */
export async function getAllSyncData(
  auth_key: string
): Promise<ServerRecord[]> {
  const res = await fetch(
    `${getBaseUrl()}/api/sync/get_all_sync_data?auth_key=${encodeURIComponent(auth_key)}`
  )
  if (!res.ok) {
    throw new Error(`Failed to fetch sync data: ${res.statusText}`)
  }
  const data: GetAllSyncDataResponse = await res.json()
  return data.all_records
}

/**
 * 上传单个记录到服务端
 */
export async function uploadRecord(
  record_id: string,
  record: ServerRecord,
  auth_key: string
): Promise<UploadRecordResponse> {
  const res = await fetch(`${getBaseUrl()}/api/sync/upload/${record_id}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ ...record, auth_key })
  })
  return res.json()
}

/**
 * 从服务端下载单条记录
 */
export async function downloadRecord(
  record_id: string,
  auth_key: string
): Promise<ServerRecord> {
  const res = await fetch(
    `${getBaseUrl()}/api/sync/download/${record_id}?auth_key=${encodeURIComponent(auth_key)}`
  )
  if (!res.ok) {
    throw new Error(`Failed to download record: ${res.statusText}`)
  }
  const data: DownloadRecordResponse = await res.json()
  return data.record
}

// ==================== 握手算法（客户端核心逻辑） ====================

/**
 * 执行握手算法，计算拉取表、推送表和冲突列表
 *
 * @param local_records 本地所有状态为 pending 的记录数组
 * @param remote_records 服务端返回的所有记录
 * @returns SyncHandshakeResult
 */
export function handshake(
  local_records: Array<
    {
      id: string
      version: number
      status: 'pending' | 'synced'
      deleted_at: number | null
    } & { [key: string]: any }
  >,
  remote_records: ServerRecord[]
): SyncHandshakeResult {
  const result: SyncHandshakeResult = {
    pull_list: [],
    push_list: [],
    conflicts: []
  }

  // 构建远程记录 Map（按 table_name + id 组合键）
  const remote_map = new Map<string, ServerRecord>()
  for (const rec of remote_records) {
    const key = `${rec.table_name}:${rec.id}`
    remote_map.set(key, rec)
  }

  // ========== c -> s: 处理本地记录 ==========
  for (const local_rec of local_records) {
    const rec_id = local_rec.id
    const local_status = local_rec.status
    const local_version = local_rec.version
    const local_deleted = local_rec.deleted_at !== null

    // 查找对应的远程记录
    const server_rec = remote_records.find((rec) => rec.id === rec_id)

    if (server_rec) {
      const server_version = server_rec.version
      const server_deleted = server_rec.deleted_at !== null

      if (local_status === 'synced' && local_version === server_version) {
        // 双方一致，无需操作
      } else if (
        local_status === 'pending' &&
        local_version === server_version
      ) {
        // 本地有修改，服务端未变化 → 推送
        result.push_list.push(rec_id)
      } else if (
        local_status === 'synced' &&
        local_version < server_version &&
        !server_deleted
      ) {
        // 本地未修改，服务端有新版本 → 拉取
        result.pull_list.push(rec_id)
      } else {
        // 本地有修改且服务端也有变化 → 冲突
        console.log(
          'Conflict local:',
          rec_id,
          local_version,
          server_version,
          local_deleted,
          server_deleted
        )
        result.conflicts.push({
          id: rec_id,
          local_version,
          server_version,
          local_deleted,
          server_deleted
        })
      }
    } else {
      // 服务端不存在 → 推送（新建记录）
      result.push_list.push(rec_id)
    }
  }

  // ========== s -> c: 处理服务端有但本地不存在的记录 ==========
  for (const server_rec of remote_records) {
    const rec_id = server_rec.id
    const remote_deleted = server_rec.deleted_at !== null

    // 查找本地记录
    const local_rec = local_records.find((r) => r.id === rec_id)

    if (!local_rec && !remote_deleted) {
      // 本地不存在且服务端未删除 → 拉取
      result.pull_list.push(rec_id)
    }
  }

  return result
}

/**
 * 应用拉取的记录到本地数据库
 * TS 根据 table_name 路由到各个实体的具体 upsert 接口
 */
const TABLE_APPLY_ORDER: Record<string, number> = {
  subjects: 0,
  error_questions: 1,
  srs_data: 2,
  error_tags: 2,
  attachments: 2,
  sources: 3
}

export async function applyPullRecords(records: ServerRecord[]): Promise<void> {
  const sorted = [...records].sort(
    (a, b) =>
      (TABLE_APPLY_ORDER[a.table_name] ?? 99) -
      (TABLE_APPLY_ORDER[b.table_name] ?? 99)
  )
  for (const rec of sorted) {
    const { table_name, id, version, status, deleted_at, data } = rec
    let input: any
    switch (table_name) {
      case 'error_questions':
        await invoke('upsert_error_question', {
          input: { id, version, status, deleted_at, ...data }
        })
        break
      case 'subjects':
        await invoke('upsert_subject', {
          input: { id, version, status, deleted_at, ...data }
        })
        break
      case 'srs_data':
        await invoke('upsert_srs_data', {
          input: { id, version, status, deleted_at, ...data }
        })
        break
      case 'attachments':
        input = { id, version, status, deleted_at, ...data }
        console.log(input)
        await invoke('upsert_attachment', {
          input: input
        })
        break
      case 'error_tags':
        input = { id, version, status, deleted_at, ...data }
        console.log(input)
        await invoke('upsert_error_tag', {
          input: { id, version, status, deleted_at, ...data }
        })
        break
      case 'sources':
        await invoke('upsert_source', {
          input: { id, version, status, deleted_at, ...data }
        })
        break
      default:
        console.warn(`Unknown table_name: ${table_name}`)
        break
    }
  }
}

// /**
//  * 批量上传记录到服务器
//  * @param records - 包含 table_name 的完整记录
//  * @param auth_key - 用户认证 key
//  */
// export async function uploadRecords(
//   records: ServerRecord[],
//   auth_key: string
// ): Promise<Map<string, boolean>> {
//   const results = new Map<string, boolean>();

//   // 并发上传，限制同时请求数
//   const concurrencyLimit = 5;

//   for (let i = 0; i < records.length; i += concurrencyLimit) {
//     const chunk = records.slice(i, i + concurrencyLimit);
//     await Promise.all(
//       chunk.map(async (rec) => {
//         try {
//           // 上传前保证状态为 synced
//           rec.status = 'synced';
//           const res = await uploadRecord(rec.id, rec, auth_key);
//           if (res.success) {
//             // 上传后更新数据库状态
//             let rst = await invoke("set_record_sync_status_version", {
//               recordId: rec.id,
//               version: res.new_version,
//               status: 'synced',
//             })
//             console.log("上传后", rst)
//           }
//           results.set(rec.id, res.success);
//         } catch (e) {
//           console.error(`Failed to upload record ${rec.id}:`, e);
//           results.set(rec.id, false);
//         }
//       })
//     );
//   }

//   return results;
// }

/**
 * 获取本地所有 pending 记录（通过 Rust invoke）
 * 返回包含 table_name 的完整记录列表
 */
export async function getLocalPendingRecords(): Promise<ServerRecord[]> {
  return invoke('get_all_pending_records')
}

/**
 * 清理所有已同步(synced)且已软删除的记录，执行真删除
 */
export async function purgeSyncedDeletions(): Promise<
  Record<string, { deleted: number }>
> {
  return invoke('purge_synced_deletions')
}

/**
 * 获取本地所有记录（无视 status），仅含握手所需字段，不含 data 负载
 */
export async function getAllLocalRecords(): Promise<SyncRecordHeader[]> {
  return invoke('get_all_records')
}

// ==================== 导出 ====================

export type {
  GetAllSyncDataResponse,
  UploadRecordResponse,
  DownloadRecordResponse
}
